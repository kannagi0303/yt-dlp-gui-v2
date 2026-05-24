use crate::app::compatibility_profiles::{
    CompatibilityScope, scope_for_target,
};
use crate::infrastructure::{
    FrameRatePolicy, ResolutionPolicy, TranscodeIntentMode, TranscodeIntentSettings,
    TranscodeSettingKey, VideoCodecPolicy,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IntentGraphNodeKind {
    Root,
    PrimaryIntent,
    TerminalControl,
    CompatibilityChoice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TranscodeGraphAxis {
    Compatibility,
    Capacity,
    Resolution,
    Format,
}

impl TranscodeGraphAxis {
    pub(super) fn variants() -> [Self; 4] {
        [Self::Compatibility, Self::Capacity, Self::Resolution, Self::Format]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CapacityChoice {
    Unlimited,
    Limited,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ResolutionChoice {
    Max1080p,
    Max720p,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FormatChoice {
    HighCompatibility,
    HighEfficiency,
    Preserve,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IntentGraphNodeId {
    Root,
    Primary(TranscodeGraphAxis),
    CapacityChoice(CapacityChoice),
    ResolutionChoice(ResolutionChoice),
    FormatChoice(FormatChoice),
    CompatibilityScope(CompatibilityScope),
}

#[derive(Clone, Debug)]
pub(super) struct IntentGraphNodeDef {
    pub id: IntentGraphNodeId,
    pub label_key: &'static str,
    pub value_text: Option<String>,
    pub kind: IntentGraphNodeKind,
    pub parent: Option<IntentGraphNodeId>,
    pub setting_key: Option<TranscodeSettingKey>,
    pub selected: bool,
}

#[derive(Clone, Debug)]
pub(super) struct IntentGraphState {
    pub selected_primary_intent: TranscodeGraphAxis,
    pub selected_child_node: Option<IntentGraphNodeId>,
    pub expanded_primary_intent: TranscodeGraphAxis,
    pub expansion_progress: f32,
    pub current_route: Vec<IntentGraphNodeId>,
}

#[derive(Clone, Debug)]
pub(super) struct IntentGraphModel {
    pub root: IntentGraphNodeDef,
    pub primary_nodes: Vec<IntentGraphNodeDef>,
    pub expanded_children: Vec<IntentGraphNodeDef>,
    pub state: IntentGraphState,
}

pub(super) fn build_intent_graph(settings: &TranscodeIntentSettings) -> IntentGraphModel {
    let selected_primary = selected_axis(settings);
    let selected_child = selected_child_for(settings, selected_primary);
    let mut current_route = vec![IntentGraphNodeId::Root, IntentGraphNodeId::Primary(selected_primary)];
    if let Some(child) = selected_child {
        current_route.push(child);
    }

    IntentGraphModel {
        root: IntentGraphNodeDef {
            id: IntentGraphNodeId::Root,
            label_key: "processing.transcode",
            value_text: None,
            kind: IntentGraphNodeKind::Root,
            parent: None,
            setting_key: None,
            selected: false,
        },
        primary_nodes: TranscodeGraphAxis::variants()
            .into_iter()
            .map(|axis| IntentGraphNodeDef {
                id: IntentGraphNodeId::Primary(axis),
                label_key: axis_label_key(axis),
                value_text: None,
                kind: IntentGraphNodeKind::PrimaryIntent,
                parent: Some(IntentGraphNodeId::Root),
                setting_key: None,
                selected: axis == selected_primary,
            })
            .collect(),
        expanded_children: children_for(selected_primary, selected_child),
        state: IntentGraphState {
            selected_primary_intent: selected_primary,
            selected_child_node: selected_child,
            expanded_primary_intent: selected_primary,
            expansion_progress: 1.0,
            current_route,
        },
    }
}

fn selected_axis(settings: &TranscodeIntentSettings) -> TranscodeGraphAxis {
    match settings.intent_mode {
        TranscodeIntentMode::DeviceCompat => TranscodeGraphAxis::Compatibility,
        TranscodeIntentMode::TargetSize | TranscodeIntentMode::ReduceSize => TranscodeGraphAxis::Capacity,
        TranscodeIntentMode::QualityFirst => TranscodeGraphAxis::Resolution,
        TranscodeIntentMode::FastTranscode => TranscodeGraphAxis::Format,
    }
}

fn selected_child_for(
    settings: &TranscodeIntentSettings,
    axis: TranscodeGraphAxis,
) -> Option<IntentGraphNodeId> {
    match axis {
        TranscodeGraphAxis::Compatibility => scope_for_target(settings.compatibility_target)
            .map(IntentGraphNodeId::CompatibilityScope),
        TranscodeGraphAxis::Capacity => Some(IntentGraphNodeId::CapacityChoice(
            if settings.size_ratio_percent >= 100 {
                CapacityChoice::Unlimited
            } else {
                CapacityChoice::Limited
            },
        )),
        TranscodeGraphAxis::Resolution => match settings.resolution_policy {
            ResolutionPolicy::Max1080p => Some(IntentGraphNodeId::ResolutionChoice(ResolutionChoice::Max1080p)),
            ResolutionPolicy::Max720p => Some(IntentGraphNodeId::ResolutionChoice(ResolutionChoice::Max720p)),
            ResolutionPolicy::AutoBalance | ResolutionPolicy::KeepOriginal => None,
        },
        TranscodeGraphAxis::Format => Some(IntentGraphNodeId::FormatChoice(match settings.video_codec_policy {
            VideoCodecPolicy::H264 => FormatChoice::HighCompatibility,
            VideoCodecPolicy::Hevc | VideoCodecPolicy::Av1 => FormatChoice::HighEfficiency,
            VideoCodecPolicy::Auto => FormatChoice::Preserve,
        })),
    }
}

fn children_for(
    axis: TranscodeGraphAxis,
    selected_child: Option<IntentGraphNodeId>,
) -> Vec<IntentGraphNodeDef> {
    match axis {
        TranscodeGraphAxis::Compatibility => CompatibilityScope::variants()
            .into_iter()
            .map(|scope| {
                let id = IntentGraphNodeId::CompatibilityScope(scope);
                IntentGraphNodeDef {
                    id,
                    label_key: "transcode.graph.compatibility_scope",
                    value_text: None,
                    kind: IntentGraphNodeKind::CompatibilityChoice,
                    parent: Some(IntentGraphNodeId::Primary(TranscodeGraphAxis::Compatibility)),
                    setting_key: Some(TranscodeSettingKey::CompatibilityTarget),
                    selected: Some(id) == selected_child,
                }
            })
            .collect(),
        TranscodeGraphAxis::Capacity => [CapacityChoice::Unlimited, CapacityChoice::Limited]
            .into_iter()
            .map(|choice| {
                let id = IntentGraphNodeId::CapacityChoice(choice);
                IntentGraphNodeDef {
                    id,
                    label_key: "transcode.graph.capacity_target",
                    value_text: None,
                    kind: IntentGraphNodeKind::TerminalControl,
                    parent: Some(IntentGraphNodeId::Primary(TranscodeGraphAxis::Capacity)),
                    setting_key: Some(TranscodeSettingKey::SizeRatio),
                    selected: Some(id) == selected_child,
                }
            })
            .collect(),
        TranscodeGraphAxis::Resolution => [ResolutionChoice::Max1080p, ResolutionChoice::Max720p]
            .into_iter()
            .map(|choice| {
                let id = IntentGraphNodeId::ResolutionChoice(choice);
                IntentGraphNodeDef {
                    id,
                    label_key: "transcode.graph.resolution_limit",
                    value_text: None,
                    kind: IntentGraphNodeKind::TerminalControl,
                    parent: Some(IntentGraphNodeId::Primary(TranscodeGraphAxis::Resolution)),
                    setting_key: Some(TranscodeSettingKey::ResolutionPolicy),
                    selected: Some(id) == selected_child,
                }
            })
            .collect(),
        TranscodeGraphAxis::Format => [
            FormatChoice::HighCompatibility,
            FormatChoice::HighEfficiency,
            FormatChoice::Preserve,
        ]
        .into_iter()
        .map(|choice| {
            let id = IntentGraphNodeId::FormatChoice(choice);
            IntentGraphNodeDef {
                id,
                label_key: "transcode.graph.format_goal",
                value_text: None,
                kind: IntentGraphNodeKind::TerminalControl,
                parent: Some(IntentGraphNodeId::Primary(TranscodeGraphAxis::Format)),
                setting_key: Some(TranscodeSettingKey::VideoCodecPolicy),
                selected: Some(id) == selected_child,
            }
        })
        .collect(),
    }
}

fn axis_label_key(axis: TranscodeGraphAxis) -> &'static str {
    match axis {
        TranscodeGraphAxis::Compatibility => "transcode.graph.axis.compatibility",
        TranscodeGraphAxis::Capacity => "transcode.graph.axis.capacity",
        TranscodeGraphAxis::Resolution => "transcode.graph.axis.resolution",
        TranscodeGraphAxis::Format => "transcode.graph.axis.format",
    }
}

#[allow(dead_code)]
fn _frame_rate_policy_used_by_graph(_: FrameRatePolicy) {}
