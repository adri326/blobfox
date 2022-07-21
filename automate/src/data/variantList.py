from dataclasses import dataclass
from typing import Final
from collections.abc import Collection, Iterator

from data.emote import Emote


# noinspection PyFinal
@dataclass(frozen=True, slots=True)
class VariantList(Collection[Emote]):
	""" An immutable variant list  """
	name: Final[ str ]
	basedOff: Final[ list[ str ] ]
	variants: Final[ tuple[Emote] ]

	def __init__( self, name: str, variants: list[ dict ], basedOff: list[str] | None = None ) -> None:
		object.__setattr__(
			self,
			'variants',
			tuple( Emote.load( variants, name ) )
		)
		object.__setattr__( self, 'basesOff', basedOff or [] )
		object.__setattr__( self, 'name', name )

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

	def __repr__( self ) -> str:
		return f'VariantList{{name={self.name}, variants={repr(self.variants)}}}'
