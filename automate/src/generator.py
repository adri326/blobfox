from pathlib import Path

import yaml
from cairosvg.surface import Surface


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

	def generate( self, outputDir: Path = Path('.') ) -> None:
		"""
		Generates the images in the given folder
		\t
		:param outputDir: output directory
		"""
		if not outputDir.exists():
			outputDir.mkdir()

		print( self.declarations['neugeme'] )


if __name__ == '__main__':
	Generator( Path('./resources/neugeme.yml') ).generate( Path('./run') )
