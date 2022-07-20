from dataclasses import dataclass
from pathlib import Path

from cairosvg.surface import Surface
from yaml import load


@dataclass(frozen=True)
class Object:
	type: str
	src: Path | None


@dataclass(frozen=True)
class Overwrite:
	id: str
	color: str | None
	remove: bool = False


@dataclass(frozen=True)
class Emote:
	name: str
	src: Path | None = None
	objects: list[Object] | None = None
	overwrite: list[Overwrite] | None = None


class Generator:

	def generate( self, declarationFile: Path, outputDir: Path = Path('.') ) -> None:
		if not declarationFile.exists():
			raise FileNotFoundError('Declaration file does not exist!')

		if not outputDir.exists():
			outputDir.mkdir()

		data = load( declarationFile.read_text(), None )
		print( data )




if __name__ == '__main__':
	gen = Generator()
	gen.generate( Path('./resources/blobfox.yml'), Path('./run') )
