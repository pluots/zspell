from zspell import Dictionary

CFG_STR = """SET UTF-8

PFX A Y 1
PFX A   0     aa         .

SFX B Y 2
SFX B   y     bb         y
SFX B   0     cc         [^y]
"""

DICT_STR = """3
xxx/A
yyy/B
zzz/AB
"""


def test_simple():
    d = Dictionary(CFG_STR, DICT_STR)
    assert d.check("xxx")
    assert d.check("aaxxx")
    assert d.check("aazzzcc")
