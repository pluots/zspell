# Configuration file for the Sphinx documentation builder.
#
# This file only contains a selection of the most common options. For a full
# list see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Path setup --------------------------------------------------------------

# If extensions (or modules to document with autodoc) are in another directory,
# add these directories to sys.path here. If the directory is relative to the
# documentation root, use os.path.abspath to make it absolute, like shown here.
#
# sys.path.insert(0, os.path.abspath('.'))

import re
from pathlib import Path

import m2r

# -- Project information -----------------------------------------------------

project = "zspell"
copyright = "2023, Trevor Gross"
author = "Trevor Gross"

# The full version, including alpha/beta/rc tags
path = Path(__file__).parent.parent.joinpath("Cargo.toml")
with path.open() as fs:
    fstr = fs.read()

# Single source of truth for the version
release = re.search(r"^version\s*=\s*\"(.*)\"$", fstr, re.MULTILINE).groups()[0]


# -- General configuration ---------------------------------------------------

# Add any Sphinx extension module names here, as strings. They can be
# extensions coming with Sphinx (named 'sphinx.ext.*') or your custom
# ones.
extensions = [
    "sphinx.ext.duration",
    "sphinx.ext.doctest",
    "sphinx.ext.autodoc",
    "sphinx.ext.autosummary",
    "sphinx.ext.intersphinx",
]


# Add any paths that contain templates here, relative to this directory.
templates_path = ["_templates"]

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
# This pattern also affects html_static_path and html_extra_path.
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

github_url = "https://github.com/pluots/zspell/"

# Autodoc options
autodoc_member_order = "bysource"
autoclass_content = "both"


# -- Options for HTML output -------------------------------------------------

# The theme to use for HTML and HTML Help pages.  See the documentation for
# a list of builtin themes.
#
html_theme = "furo"

# Add any paths that contain custom static files (such as style sheets) here,
# relative to this directory. They are copied after the builtin static files,
# so a file named "default.css" will overwrite the builtin "default.css".
html_static_path = ["_static"]


def convert_docstrings(app, what, name, obj, options, lines):
    """Convert docstrings from markdown to RST"""
    md = "\n".join(lines)
    rst = m2r.convert(md)
    lines.clear()
    lines += rst.splitlines()


def setup(app):
    app.connect("autodoc-process-docstring", convert_docstrings)
