from dataclasses import dataclass
from pathlib import Path
from typing_extensions import Self


@dataclass(frozen=True)
class Object:
	type: str
	src: Path | None

	@classmethod
	def load( cls, data: list[dict] ) -> list[ Self ]:
		objects: list[ Object ] = []
		for obj in data or []:
			objects += [ Object( **obj ) ]

		return objects
