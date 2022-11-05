import multiprocessing

import pytest
from test_matcher_rs import TextMatcher

DATA_DIR = "./test_data/plzs"


def assert_expected_match(matcher: TextMatcher, text: str, expected: str):
    matches = matcher.find_matches(text)

    print(text, expected)
    assert len(matches) != 0
    assert matches[0].text == expected


@pytest.mark.parametrize(
    "candidates_file,text,expected",
    [
        (f"{DATA_DIR}/1201", "qu du seujet 36", "quai du seujet 36"),
        (
            f"{DATA_DIR}/1000",
            "rt de la clai-au-moin 21",
            "route de la claie-aux-moines 21",
        ),
        (
            f"{DATA_DIR}/1000",
            "rt de la claie 21",
            "route de la claie-aux-moines 21",
        ),
        (
            f"{DATA_DIR}/1000",
            "chemin de la planche 9a",
            "chemin de la planche-aux-oies 9a",
        ),
        (f"{DATA_DIR}/9650", "hagenstr 5", "haggenstrasse 5"),
        (f"{DATA_DIR}/1753", "impasse de pra damont 2", "impasse de pra d'amont 2"),
        (f"{DATA_DIR}/1753", "imp de pra damont 2", "impasse de pra d'amont 2"),
        (
            f"{DATA_DIR}/1753",
            "rt de villars-sur-glane 2",
            "route de villars-sur-glâne 2",
        ),
        (f"{DATA_DIR}/1945", "rt de riere-ville 10.afsdfsf", "route de rière-ville 10"),
    ],
)
def test_different_files(candidates_file: str, text: str, expected: str):
    # NOTE: sensitivity value is really critical, so it's better to keep it low
    # if you work with the real-time text processing
    assert_expected_match(
        TextMatcher(sensitivity=0.1, keep=1, path_to_candidates=candidates_file),
        text,
        expected,
    )


@pytest.mark.parametrize(
    "text,expected",
    [
        ("qu du seujet 36", "quai du seujet 36"),
        (
            "rt de la claie-aux-moine 21",
            "route de la claie-aux-moines 21",
        ),
        (
            "rt de la claie-aux-moine 21",
            "route de la claie-aux-moines 21",
        ),
        (
            "ch de la planche-aux-oies 9a",
            "chemin de la planche-aux-oies 9a",
        ),
        ("haggenstr 5", "haggenstrasse 5"),
        ("impasse de pra damont 2", "impasse de pra d'amont 2"),
        ("imp de pra damont 2", "impasse de pra d'amont 2"),
        (
            "rt de villars-sur-glane 2",
            "route de villars-sur-glâne 2",
        ),
        ("rt de riere-ville 10.afsdfsf", "route de rière-ville 10"),
    ],
)
def test_multiple_files(text: str, expected: str):
    matches = TextMatcher.find_matches_in_dir(
        sens=0.1,
        keep=1,
        text=text,
        path_to_dir=DATA_DIR,
        num_of_threads=multiprocessing.cpu_count(),
    )
    assert len(matches) != 0
    assert matches[0].text == expected


def test_high_sensitivity():
    matcher = TextMatcher(0.99, 5, f"{DATA_DIR}/1201")
    assert not matcher.find_matches("qu du seujet 36")


def test_zero_to_keep():
    matcher = TextMatcher(0.1, 0, f"{DATA_DIR}/1201")
    assert not matcher.find_matches("qu du seujet 36")


def test_normal_sensitivity():
    matcher = TextMatcher(0.7, 5, f"{DATA_DIR}/1201")
    assert matcher.find_matches("qu du seujet 36")[0].text == "quai du seujet 36"
