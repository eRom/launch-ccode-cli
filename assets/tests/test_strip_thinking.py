"""Tests unitaires pour le callback strip_thinking."""

from types import SimpleNamespace

from lcc_strip_thinking import _strip_thinking_blocks


def make_response(content: list) -> SimpleNamespace:
    return SimpleNamespace(content=content)


def test_strips_unsigned_thinking_block():
    resp = make_response([
        {"type": "thinking", "thinking": "raisonnement…"},
        {"type": "text", "text": "Hello!"},
    ])
    _strip_thinking_blocks(resp)
    assert resp.content == [{"type": "text", "text": "Hello!"}]


def test_keeps_signed_thinking_block():
    resp = make_response([
        {"type": "thinking", "thinking": "…", "signature": "sig-abc"},
        {"type": "text", "text": "yo"},
    ])
    _strip_thinking_blocks(resp)
    assert len(resp.content) == 2
    assert resp.content[0]["type"] == "thinking"


def test_handles_no_content_attribute():
    resp = SimpleNamespace()  # pas de .content
    _strip_thinking_blocks(resp)  # ne doit pas crasher


def test_handles_non_list_content():
    resp = make_response("plain string")
    _strip_thinking_blocks(resp)
    assert resp.content == "plain string"  # inchangé


def test_empty_after_strip_is_ok():
    resp = make_response([
        {"type": "thinking", "thinking": "…"},
    ])
    _strip_thinking_blocks(resp)
    assert resp.content == []
