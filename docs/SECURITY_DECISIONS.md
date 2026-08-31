# Security decisions

This document owns durable repository-specific security decisions: the selected
boundary, restrictive configuration, rationale, and constraint future changes
preserve. General technical choices remain in
[DESIGN_DECISIONS.md](DESIGN_DECISIONS.md). This is not a vulnerability-reporting
policy or a claim that Ferrum is release-ready.

## XML parsing

### Python XML parsers use restrictive lxml configuration

**Decision.** Current Python tests and E2E tools parse XML with
`lxml.etree.XMLParser`, not `defusedxml`. The parser class alone is not the
security guarantee. Every parser instance uses this restrictive configuration:

```python
_XML_PARSER = lxml.etree.XMLParser(
	load_dtd=False,
	resolve_entities=False,
	no_network=True,
	recover=False,
	huge_tree=False,
)
```

Callers provide consistently encoded bytes. Any genuinely untrusted ingress
also bounds input size before parsing and keeps `lxml` and `libxml2` patched.
When an owning format forbids `DOCTYPE`, its ingress rejects that syntax rather
than relying only on disabled entity resolution. This configuration satisfies
ASVS 1.5.1 by disabling unsafe XML features, including external-entity
resolution.

**Why.** Disabling DTD loading, entity resolution, network access, error
recovery, and oversized-tree support prevents Python tooling from silently
expanding external content or accepting a more permissive XML language. One
configured parser dependency also prevents behavior and dependency policy from
drifting between Python consumers.

**Consequence.** Do not add `defusedxml` or use a bare/default XML parser in
current Python source or tests. Reuse the explicit restrictive `lxml` pattern.
Production CDML parsing and admission remain Rust/`xot` responsibilities rather
than a Python parser boundary.

**Owner.** [CDML_FORMAT_SPEC.md](CDML_FORMAT_SPEC.md),
[e2e_atom_label_bond_alignment.py](../tests/e2e/e2e_atom_label_bond_alignment.py),
and [pip_requirements-dev.txt](../pip_requirements-dev.txt).
