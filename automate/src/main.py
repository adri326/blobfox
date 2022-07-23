import os
import sys
from argparse import ArgumentParser
from pathlib import Path
from typing import cast

from generator import Generator

parser = ArgumentParser(
	prog=( 'automation' + '.exe' if 'win' in os.name.lower() else '' ) if getattr( sys, 'frozen', False ) else 'main.py',
	description='Generates blobfoxes from yaml and svgs'
)

parser.add_argument(
	'-d',
	'--declfile',
	help='Declaration file to export',
	action='store',
	type=Path,
	dest='declFile',
	required=True
)
parser.add_argument(
	'-e',
	'--export',
	help='A comma-separated list of emote to export',
	action='store',
	dest='exports',
	default='<all>'
)
parser.add_argument(
	'-o',
	'--output',
	help='Output directory',
	action='store',
	type=Path,
	dest='output',
	default=Path('.')
)


class Arguments:
	declFile: Path
	exports: str
	output: Path


if __name__ == '__main__':
	args: Arguments = cast( parser.parse_args( sys.argv[1:] ), Arguments )

	gen = Generator( args.declFile )

	gen.export( args.exports.split(','), args.output )


