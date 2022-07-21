from __future__ import annotations

import typing
from dataclasses import dataclass
from pathlib import Path
from typing import Final
from collections.abc import Collection, Iterator

import util
from data.emote import Emote


if typing.TYPE_CHECKING:
	from generator import Generator


# noinspection PyFinal
@dataclass(frozen=True, slots=True)
class Variants( Collection[Emote] ):
	""" An immutable variant list  """
	name: Final[ str ]
	basedOff: Final[ tuple[ str ] ]
	variants: Final[ tuple[ Emote ] ]

	def __init__( self, path: Path, gen: Generator ) -> None:
		data = util.load( path )

		object.__setattr__( self, 'name', data[ 'name' ] )
		object.__setattr__( self, 'basedOff', tuple( data[ 'basedOff' ] or [ ] ) )

		if data['basedOff']:
			# loading a set based off another, load the base one before this
			base = gen.load( path.parent / f'{data["basedOff"]}.yml' )

			# additional emotes may be defined
			additional = [ ]
			# apply overwrites and append new emotes to `additional`
			for emote in Emote.load( data['variants'], self.name ):
				if emote.name in base:
					base._applyOverwrite( emote )
				else:
					additional += [ emote ]
			# save the newly created set of emotes
			object.__setattr__( self, 'variants', base.variants + tuple( additional ) )
		else:
			# loading a baseless set, just load it directly
			object.__setattr__( self, 'variants', tuple( Emote.load( data['variants'], self.name ) ) )

	def __iter__( self ) -> Iterator[Emote ]:
		return self.variants.__iter__()

	def __contains__( self, item: object ) -> bool:
		if isinstance( item, str ):
			for elem in self.variants:
				if elem.name == item:
					return True
			return False
		return item in self.variants

	def __getitem__( self, item: int | str ) -> Emote:
		if isinstance( item, int ):
			return self.variants[ item ]

		if isinstance( item, str ):
			for elem in self.variants:
				if elem.name == item:
					return elem
			raise KeyError( f'A variant with name "{item}" does not exist' )

		raise ValueError( f'Invalid __getitem__ input: {item}' )

	def __len__( self ) -> int:
		return len( self.variants )

	def _applyOverwrite( self, overwriter: Emote ) -> None:
		emote = self[ overwriter.name ]
		emote.origin += overwriter.origin
		emote.overwrites += overwriter.overwrites
		emote.objects += overwriter.objects

