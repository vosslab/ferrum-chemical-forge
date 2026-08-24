/// Closed semantic request for one direct-root presentation record.
///
/// The document session allocates the durable identity and admits the candidate
/// through the generic transition lifecycle. Callers provide only the authored
/// geometry and closed presentation semantics.
#[derive(Clone, Debug, PartialEq)]
pub enum CreatePresentationRootV1 {
    StraightNormalArrow {
        start: crate::PresentationGesturePoint2V1,
        end: crate::PresentationGesturePoint2V1,
        style: crate::ArrowGestureStyleV1,
    },
    StraightEquilibriumArrow {
        start: crate::PresentationGesturePoint2V1,
        end: crate::PresentationGesturePoint2V1,
    },
    StandardPlus {
        anchor: crate::PresentationGesturePoint2V1,
    },
}

impl CreatePresentationRootV1 {
    #[must_use]
    pub const fn straight_normal_arrow(
        start: crate::PresentationGesturePoint2V1,
        end: crate::PresentationGesturePoint2V1,
        style: crate::ArrowGestureStyleV1,
    ) -> Self {
        Self::StraightNormalArrow { start, end, style }
    }

    #[must_use]
    pub const fn straight_equilibrium_arrow(
        start: crate::PresentationGesturePoint2V1,
        end: crate::PresentationGesturePoint2V1,
    ) -> Self {
        Self::StraightEquilibriumArrow { start, end }
    }

    #[must_use]
    pub const fn standard_plus(anchor: crate::PresentationGesturePoint2V1) -> Self {
        Self::StandardPlus { anchor }
    }
}

/// Versioned session operation staging the initial supported document mutation.
#[derive(Clone, Debug, PartialEq)]
pub struct CreateCurvedTerminalArrowV1 {
    kind: crate::CurvedTerminalArrowKindV1,
    start: crate::PresentationGesturePoint2V1,
    control: crate::PresentationGesturePoint2V1,
    end: crate::PresentationGesturePoint2V1,
}

impl CreateCurvedTerminalArrowV1 {
    #[must_use]
    pub const fn new(
        kind: crate::CurvedTerminalArrowKindV1,
        start: crate::PresentationGesturePoint2V1,
        control: crate::PresentationGesturePoint2V1,
        end: crate::PresentationGesturePoint2V1,
    ) -> Self {
        Self {
            kind,
            start,
            control,
            end,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> crate::CurvedTerminalArrowKindV1 {
        self.kind
    }
    #[must_use]
    pub const fn start(&self) -> crate::PresentationGesturePoint2V1 {
        self.start
    }
    #[must_use]
    pub const fn control(&self) -> crate::PresentationGesturePoint2V1 {
        self.control
    }
    #[must_use]
    pub const fn end(&self) -> crate::PresentationGesturePoint2V1 {
        self.end
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateCurvedEquilibriumArrowV1 {
    start: crate::PresentationGesturePoint2V1,
    control: crate::PresentationGesturePoint2V1,
    end: crate::PresentationGesturePoint2V1,
}

impl CreateCurvedEquilibriumArrowV1 {
    #[must_use]
    pub const fn new(
        start: crate::PresentationGesturePoint2V1,
        control: crate::PresentationGesturePoint2V1,
        end: crate::PresentationGesturePoint2V1,
    ) -> Self {
        Self {
            start,
            control,
            end,
        }
    }
    #[must_use]
    pub const fn start(&self) -> crate::PresentationGesturePoint2V1 {
        self.start
    }
    #[must_use]
    pub const fn control(&self) -> crate::PresentationGesturePoint2V1 {
        self.control
    }
    #[must_use]
    pub const fn end(&self) -> crate::PresentationGesturePoint2V1 {
        self.end
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreatePresentationPathV1 {
    path: crate::PresentationPathGestureV1,
    appearance: crate::PresentationAppearanceV1,
}

impl CreatePresentationPathV1 {
    #[must_use]
    pub const fn new(
        path: crate::PresentationPathGestureV1,
        appearance: crate::PresentationAppearanceV1,
    ) -> Self {
        Self { path, appearance }
    }
    #[must_use]
    pub const fn path(&self) -> &crate::PresentationPathGestureV1 {
        &self.path
    }
    #[must_use]
    pub const fn appearance(&self) -> &crate::PresentationAppearanceV1 {
        &self.appearance
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreatePresentationVectorV1 {
    kind: crate::PresentationVectorCreateKindV1,
    start: crate::PresentationGesturePoint2V1,
    end: crate::PresentationGesturePoint2V1,
    appearance: crate::PresentationAppearanceV1,
}

impl CreatePresentationVectorV1 {
    #[must_use]
    pub const fn new(
        kind: crate::PresentationVectorCreateKindV1,
        start: crate::PresentationGesturePoint2V1,
        end: crate::PresentationGesturePoint2V1,
        appearance: crate::PresentationAppearanceV1,
    ) -> Self {
        Self {
            kind,
            start,
            end,
            appearance,
        }
    }
    #[must_use]
    pub const fn kind(&self) -> crate::PresentationVectorCreateKindV1 {
        self.kind
    }
    #[must_use]
    pub const fn start(&self) -> crate::PresentationGesturePoint2V1 {
        self.start
    }
    #[must_use]
    pub const fn end(&self) -> crate::PresentationGesturePoint2V1 {
        self.end
    }
    #[must_use]
    pub const fn appearance(&self) -> &crate::PresentationAppearanceV1 {
        &self.appearance
    }
}
