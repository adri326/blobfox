from dataclasses import dataclass
from pathlib import Path
from typing_extensions import Self

from data.object import Object
from data.overwrite import Overwrite


@dataclass
class Emote:
	name: str
	origin: list[ str ]  #: the origins in order of definition
	base: str | None = None
	src: Path | None = None
	objects: list[Object] = None
	overwrites: list[Overwrite] = None

	@classmethod
	def load( cls, data: list[dict], origin: str ) -> list[Self]:
		return [
			Emote(
				origin=[ origin ],
				**entry | {
					'overwrites': Overwrite.load( entry.get( 'overwrites', [ ] ) ),
					'objects': Object.load( entry.get( 'objects', [ ] ) )
				}
			) for entry in data
		]
