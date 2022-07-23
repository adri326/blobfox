from pathlib import Path

from PIL.Image import Image
from cairosvg.surface import Surface

from data.emote import Emote
from data.variants import Variants


class Generator:
	declarations: dict[ str, Variants ]
	surfaces: dict[ str, Surface ]

	def __init__( self, declFile: Path ) -> None:
		if not declFile.exists():
			raise FileNotFoundError('Declaration file does not exist!')

		self.declarations = { declFile.name[:-4]: self.load( declFile ) }

	def load( self, declFile: Path ) -> Variants:
		return Variants( declFile, self )

	def generate( self, emote: Emote ) -> Image:
		pass

	def export( self, sets: list[str], outputDir: Path = Path( '.' ) ) -> None:
		"""
		Generates the images in the given folder
		\t
		:param sets: the sets to export to PNGs
		:param outputDir: output directory
		"""
		if not outputDir.exists():
			outputDir.mkdir()

		for set in sets:
			# for emote in emotes
			pass


