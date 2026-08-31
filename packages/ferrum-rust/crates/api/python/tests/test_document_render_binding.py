"""Installed-wheel contracts for Ferrum's closed render observation boundary."""

import pytest

import ferrum_chem


SOURCE = (
    "<cdml xmlns='urn:ferrum:cdml'><molecule id=\"m\"><atom id=\"a\" name=\"C\">"
    "<point x=\"1\" y=\"2\"/></atom></molecule></cdml>"
)


def test_render_observation_is_one_frozen_api_owned_plan_with_exact_glyphs() -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)
    observation = session.observe_render(0)
    entry = observation.molecule_plans[0]
    plan = entry.plan
    batch = plan.batches[0]
    content = batch.content
    label = content.label
    operation = content.operations[-1]
    assert (observation.document.snapshot.revision, observation.document.snapshot.digest) == (
        plan.provenance.revision,
        plan.provenance.digest,
    )
    assert isinstance(observation.molecule_plans, tuple)
    assert (observation.schema, plan.schema, type(plan), type(batch), type(content)) == (
        "ferrum-document-render-observation-v2",
        "ferrum-render-plan-v4",
        ferrum_chem.RenderPlanV4,
        ferrum_chem.RenderBatchV4,
        ferrum_chem.AtomRenderBatchV1,
    )
    assert isinstance(content.operations, tuple)
    assert isinstance(content.decorations, tuple)
    assert content.decorations == ()
    assert (content.kind, content.atom_local_anchor.x, content.atom_local_anchor.y) == (
        "atom", 1.0, 2.0,
    )
    assert (type(label), label.core_element_run_index, type(label.text)) == (
        ferrum_chem.AtomLabelRenderV1, 0, ferrum_chem.TextOpV1,
    )
    assert (type(label.full_ink_bounds), type(label.core_element_ink_bounds)) == (
        ferrum_chem.InkBoundsV1, ferrum_chem.InkBoundsV1,
    )
    assert type(label.bond_ink_clearance) is float and label.bond_ink_clearance > 0.0
    assert (label.core_element_ink_bounds.min_x + label.core_element_ink_bounds.max_x) == 0.0
    assert (label.core_element_ink_bounds.min_y + label.core_element_ink_bounds.max_y) == 0.0
    assert type(operation) is ferrum_chem.RenderOperationV3
    paint = operation.operation.paint
    assert (type(paint), paint.kind, paint.export_rgb, paint.role, paint.element) == (
        ferrum_chem.RenderPaintV3, "theme_role", "000000", "document_foreground", None,
    )
    assert entry.molecule.document_object_id == (
        observation.document.projection.molecules[0].document_object_id
    )
    assert (type(entry.bounds), entry.bounds.left < entry.bounds.right,
        entry.bounds.top < entry.bounds.bottom) == (ferrum_chem.MoleculeContentBoundsV1, True, True)
    assert batch.target.kind == "document_object"
    assert operation.kind == "text"
    assert operation.operation.runs[0].glyphs[0].glyph_index > 0
    assert not hasattr(ferrum_chem, "RenderPlanV3")
    assert not hasattr(ferrum_chem, "RenderBatchV3")
    assert not hasattr(ferrum_chem, "RenderObservationV1")
    with pytest.raises(AttributeError):
        plan.provenance.revision = 1


def test_render_observation_preserves_exact_multi_batch_paint_order() -> None:
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
        '<atom id="c" name="C"><point x="0" y="0"/></atom>'
        '<atom id="o" name="O"><point x="20" y="0"/></atom>'
        '<bond id="co" start="c" end="o" type="n1"/>'
        '</molecule></cdml>'
    )
    session = ferrum_chem.DocumentSession.load(source)
    batches = session.observe_render(0).molecule_plans[0].plan.batches

    assert tuple((batch.paint_order, batch.content.kind) for batch in batches) == (
        (0, "atom"),
        (1, "atom"),
        (2, "bond"),
    )
    first, second, bond_batch = batches
    bond = bond_batch.content
    axis = bond.attachment_axis
    assert (type(axis), axis.start.x, axis.start.y, axis.end.x, axis.end.y) == (
        ferrum_chem.BondAttachmentAxisV1, 0.0, 0.0, 20.0, 0.0,
    )
    assert (axis.start.x, axis.start.y) == (
        first.content.atom_local_anchor.x, first.content.atom_local_anchor.y,
    )
    assert (axis.end.x, axis.end.y) == (
        second.content.atom_local_anchor.x, second.content.atom_local_anchor.y,
    )
    with pytest.raises(AttributeError):
        axis.start = first.content.atom_local_anchor


def test_render_observation_preserves_plan_issue_paint_order_and_interleaving() -> None:
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
        '<atom id="c" name="C"><point x="0" y="0"/></atom>'
        '<atom id="hidden" name="O" show="no"><point x="20" y="0"/></atom>'
        '<atom id="n" name="N"><point x="40" y="0"/></atom>'
        '<bond id="hidden-bond" start="c" end="hidden" type="n1"/>'
        '</molecule></cdml>'
    )
    session = ferrum_chem.DocumentSession.load(source)
    plan = session.observe_render(0).molecule_plans[0].plan

    assert tuple(batch.paint_order for batch in plan.batches) == (0, 2)
    assert tuple((issue.paint_order, issue.kind) for issue in plan.issues) == (
        (1, "unsupported_feature"),
        (3, "unrenderable_target"),
    )
    assert tuple(
        (paint_order, source)
        for paint_order, source in sorted(
            [(batch.paint_order, "batch") for batch in plan.batches]
            + [(issue.paint_order, "issue") for issue in plan.issues]
        )
    ) == ((0, "batch"), (1, "issue"), (2, "batch"), (3, "issue"))


def test_render_observation_transports_closed_atom_number_decoration_and_replay() -> None:
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
        '<atom id="c" name="C"><point x="0" y="0"/></atom></molecule></cdml>'
    )
    session = ferrum_chem.DocumentSession.load(source)
    observation = session.observe(0)
    molecule = observation.projection.molecules[0]
    atom = molecule.atoms[0]
    numbered = session.set_atom_number_v1(
        observation.snapshot.revision,
        observation.snapshot.digest,
        molecule.document_object_id,
        atom.document_object_id,
        7,
        True,
    ).observation
    content = session.observe_render(numbered.snapshot.revision).molecule_plans[0].plan.batches[0].content

    assert type(content) is ferrum_chem.AtomRenderBatchV1
    assert len(content.decorations) == 1
    decoration = content.decorations[0]
    assert (type(decoration), decoration.kind, type(decoration.operation)) == (
        ferrum_chem.AtomDecorationRenderOpV1,
        "text",
        ferrum_chem.TextOpV1,
    )
    assert tuple(run.text for run in decoration.operation.runs) == ("7",)
    assert tuple(operation.kind for operation in content.operations) == ("text", "text")
    assert content.operations[-1].operation is not decoration.operation
    assert tuple(run.text for run in content.operations[-1].operation.runs) == ("7",)


def test_render_observation_preserves_closed_atom_labels_for_c_o_cl_and_nh3() -> None:
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
        '<atom id="c" name="C"><point x="0" y="0"/></atom>'
        '<atom id="o" name="O"><point x="20" y="0"/></atom>'
        '<atom id="cl" name="Cl"><point x="40" y="0"/></atom>'
        '<atom id="n" name="N" charge="1" explicit_hydrogens="3" hydrogens="on">'
        '<point x="60" y="0"/></atom></molecule></cdml>'
    )
    session = ferrum_chem.DocumentSession.load(source)
    contents = tuple(
        batch.content for batch in session.observe_render(0).molecule_plans[0].plan.batches
        if type(batch.content) is ferrum_chem.AtomRenderBatchV1
    )
    labels = {
        "".join(run.text for run in content.label.text.runs): content.label
        for content in contents
    }

    assert {"C", "O", "Cl", "NH3+"} <= labels.keys()
    assert all(type(label) is ferrum_chem.AtomLabelRenderV1 for label in labels.values())
    assert all(
        type(label.core_element_run_index) is int
        and type(label.bond_ink_clearance) is float
        and label.bond_ink_clearance > 0.0
        and type(label.full_ink_bounds) is ferrum_chem.InkBoundsV1
        and type(label.core_element_ink_bounds) is ferrum_chem.InkBoundsV1
        for label in labels.values()
    )


def test_render_targets_publish_visual_and_durable_document_identities() -> None:
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="molecule">'
        '<atom id="atom" name="C"><point x="0" y="0"/></atom>'
        '<compact-group id="group" version="1" catalog-key="methyl" attachment-index="0" '
        'orientation-degrees="0"><point x="80" y="0"/></compact-group>'
        '<bond id="bond" start="atom" end="group" type="n1"/>'
        '</molecule></cdml>'
    )
    session = ferrum_chem.DocumentSession.load(source)
    observation = session.observe(0)
    entry = session.observe_render(0).molecule_plans[0]
    targets = tuple(batch.target for batch in entry.plan.batches)
    molecule = observation.projection.molecules[0]
    member_ids = {
        *(atom.document_object_id for atom in molecule.atoms),
        *(bond.document_object_id for bond in molecule.bonds),
        *(group.document_object_id for group in molecule.compact_groups),
    }

    assert targets
    assert all(target.kind == "document_object" for target in targets)
    assert {target.document_object_id for target in targets} <= member_ids


def test_render_observation_keeps_compact_group_and_bond_operations_closed() -> None:
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="molecule">'
        '<atom id="atom" name="C"><point x="0" y="0"/></atom>'
        '<compact-group id="group" version="1" catalog-key="methyl" attachment-index="0" '
        'orientation-degrees="0"><point x="80" y="0"/></compact-group>'
        '<bond id="bond" start="atom" end="group" type="n1"/>'
        '</molecule></cdml>'
    )
    session = ferrum_chem.DocumentSession.load(source)
    batches = session.observe_render(0).molecule_plans[0].plan.batches
    group = next(batch.content for batch in batches if batch.content.kind == "compact_group")
    bond = next(batch.content for batch in batches if batch.content.kind == "bond")

    assert type(group) is ferrum_chem.CompactGroupRenderBatchV1
    assert type(bond) is ferrum_chem.BondRenderBatchV1
    assert group.typed_operations
    assert bond.typed_operations
    assert all(type(value) is ferrum_chem.CompactGroupRenderOpV1 for value in group.typed_operations)
    assert all(type(value) is ferrum_chem.BondRenderOpV1 for value in bond.typed_operations)
    assert tuple((value.kind, type(value.operation)) for value in group.operations) == tuple(
        (value.kind, type(value.operation)) for value in group.typed_operations
    )
    assert tuple((value.kind, type(value.operation)) for value in bond.operations) == tuple(
        (value.kind, type(value.operation)) for value in bond.typed_operations
    )


def test_render_observation_preserves_typed_stale_and_closed_molecule_label_contracts() -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)
    resource = ferrum_chem.molecule_label_font()
    assert isinstance(resource.data, bytes)
    assert (resource.resource_id, resource.byte_length, resource.family) == (
        "ferrum-atkinson-hyperlegible-next-regular-2.001",
        len(resource.data),
        "Atkinson Hyperlegible Next",
    )
    with pytest.raises(ferrum_chem.RevisionConflictError):
        session.observe_render(1)


def test_direct_text_projection_and_render_keep_closed_runs_and_exact_glyphs() -> None:
    session = ferrum_chem.DocumentSession.load(
        '<cdml xmlns="urn:ferrum:cdml"><text id="label"><point x="10" y="20"/>'
        '<font size="18" color="#123456"/>'
        '<ftext>Line one\nH&lt;sub&gt;2&lt;/sub&gt;O</ftext></text></cdml>',
    )
    observation = session.observe_render(0)
    root = observation.document.projection.presentation_stack.entries[0]
    render = observation.text_renders[0]

    assert root.kind == "text"
    assert isinstance(root.text.runs, tuple)
    assert [(run.text, run.styles) for run in root.text.runs] == [
        ("Line one\nH", ()),
        ("2", ("subscript",)),
        ("O", ()),
    ]
    assert isinstance(render.source_runs, tuple)
    assert [run.script for run in render.source_runs] == [
        "baseline", "subscript", "baseline",
    ]
    assert isinstance(render.operation.runs, tuple)
    assert all(glyph.glyph_index > 0 for run in render.operation.runs for glyph in run.glyphs)
    assert (render.target.document_object_id, render.anchor.x, render.anchor.y) == (
        root.text.target.document_object_id,
        10.0,
        20.0,
    )
    with pytest.raises(AttributeError):
        render.operation.paint.export_rgb = "000000"
