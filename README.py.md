# ZSpell

Python bindings for the Rust zspell library: a simple yet fast spellchecker.

To use this library, you will need a dictionary in the language of your choice.
Many are available at the following repository:
<https://github.com/wooorm/dictionaries>

The full Python API documentation is available at <https://zspell.readthedocs.io>

```py
from zspell import Dictionary

with open ("dictionaries/en_US.aff", "r") as f:
    config_str = f.read()
with open ("dictionaries/en_US.dic", "r") as f:
    dict_str = f.read()
d = Dictionary(config_str, dict_str)

assert(d.check("Apples are good! Don't you think?"))
assert(not d.check("Apples are baaaad"))
```
