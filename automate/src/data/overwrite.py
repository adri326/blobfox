from dataclasses import dataclass
from typing_extensions import Self


@dataclass(frozen=True)
class Overwrite:
	id: str
	color: str | None
	remove: bool = False

	@classmethod
	def load( cls, data: list[dict] ) -> list[Self]:
		overwrites: list[ Overwrite ] = [ ]
		for overwrite in data or [ ]:
			overwrites += [ Overwrite( **overwrite ) ]

		return overwrites
