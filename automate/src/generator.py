from pathlib import Path

import yaml
from cairosvg.surface import Surface


from data.variantList import VariantList


class Generator:
	declarations: dict[ str, VariantList ]
	surfaces: dict[ str, Surface ]

	def __init__( self, declFile: Path ) -> None:
		if not declFile.exists():
			raise FileNotFoundError('Declaration file does not exist!')

		self.recursiveLoad( declFile )

	def recursiveLoad( self, declFile: Path ) -> None:
		variants = VariantList( **yaml.load( declFile.read_text(), yaml.FullLoader ) )
		self.declarations[ variants.name ] = variants

		if variants.basedOff is not None:
			self.recursiveLoad( declFile.parent / f'{variants.basedOff}.yml' )

	def generate( self, outputDir: Path = Path('.') ) -> None:
		"""
		Generates the images in the given folder
		\t
		:param outputDir: output directory
		"""
		if not outputDir.exists():
			outputDir.mkdir()

		for variant in self.variants:
			pass


if __name__ == '__main__':
	Generator( Path('./resources/blobfox.yml') ).generate( Path('./run') )
