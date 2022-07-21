from pathlib import Path
from typing import Any

import yaml


def load( path: Path ) -> Any:
	""" Parse the first YAML document in a stream and produce the corresponding Python object. """
	return yaml.load( path.read_text(), yaml.FullLoader )


def loads( data: str ) -> Any:
	""" Parse the first YAML document in a string and produce the corresponding Python object. """
	return yaml.load( data, yaml.FullLoader )
