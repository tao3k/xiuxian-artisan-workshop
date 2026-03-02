"""Tests for knowledge ingest_document URL support (download to project data)."""

from _module_loader import load_script_module


def _import_graph_module():
    return load_script_module("graph", alias="knowledge_graph_ingest_url_test")


def test_is_url():
    """_is_url identifies http(s) URLs."""
    graph = _import_graph_module()
    assert graph._is_url("https://arxiv.org/pdf/2601.03192") is True
    assert graph._is_url("http://example.com/doc.pdf") is True
    assert graph._is_url("  https://a.b/c  ") is True
    assert graph._is_url("/local/path/doc.pdf") is False
    assert graph._is_url("docs/guide.pdf") is False
    assert graph._is_url("") is False
    assert graph._is_url(None) is False


def test_filename_from_url_arxiv():
    """_filename_from_url derives arxiv ID as filename."""
    graph = _import_graph_module()
    assert graph._filename_from_url("https://arxiv.org/pdf/2601.03192") == "2601.03192.pdf"
    assert graph._filename_from_url("https://arxiv.org/pdf/2510.12323.pdf") == "2510.12323.pdf"


def test_filename_from_url_generic():
    """_filename_from_url uses path basename or document.pdf."""
    graph = _import_graph_module()
    assert graph._filename_from_url("https://example.com/papers/report.pdf") == "report.pdf"
    assert graph._filename_from_url("https://example.com/papers/report") == "report.pdf"
    assert graph._filename_from_url("https://example.com/") == "document.pdf"
