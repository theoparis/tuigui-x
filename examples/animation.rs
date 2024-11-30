use tuigui::{
	animations,
	widgets::{
		self,
		animation_container::{AnimationContainer, AnimationContainerSequence},
		label::{Label, LabelStyle},
		layers::Layers,
		widget_dyn::WidgetDyn,
		widget_pointer::WidgetPointer,
	},
	AnsiColor, Content, Context, ContextConfig, ContextSetupConfig, Position,
	Size, Style, Transform,
};
use tuigui_x::{XBackend, XContentProcessor};

// Amount of time before the demo quits.
const TIME_LIMIT: f64 = 6.0;

// Play with this to see that a lower FPS doesn't affect the
// animation's speed. (It simply just makes it choppy.)
const FRAME_DELAY: f64 = 0.0;

// The amount of seconds the animations in this demo will take to finish.
const ANIMATION_SECS: f64 = 1.0;

fn pane(color: AnsiColor) -> widgets::fill::FillArea {
	widgets::fill::FillArea::new(Some(Content::Styled(
		'█',
		Style::new().fg(Some(color)),
	)))
}

fn main() {
	// Play with me to change what custom lerp animation gets used.
	let lerp_anim = animations::linear;

	let root_1 = AnimationContainer::new(
		AnimationContainerSequence::Many(vec![
			lerp_anim(
				Transform::new(Position::new(0, 0), Size::new(20, 10)),
				ANIMATION_SECS,
			),
			lerp_anim(
				Transform::new(Position::new(40, 20), Size::new(10, 5)),
				ANIMATION_SECS,
			),
			lerp_anim(
				Transform::new(Position::new(80, 0), Size::new(20, 5)),
				ANIMATION_SECS,
			),
			lerp_anim(
				Transform::new(Position::new(100, 20), Size::new(40, 10)),
				ANIMATION_SECS,
			),
			lerp_anim(
				Transform::new(Position::new(120, 0), Size::new(20, 10)),
				ANIMATION_SECS,
			),
		]),
		true,
		pane(AnsiColor::Green),
	);

	let root_2 = AnimationContainer::new(
		AnimationContainerSequence::Single(lerp_anim(
			Transform::new(Position::new(60, 10), Size::new(30, 10)),
			ANIMATION_SECS,
		)),
		true,
		pane(AnsiColor::Red),
	);

	let root_3 = AnimationContainer::new(
		AnimationContainerSequence::Many(vec![
			lerp_anim(
				Transform::new(Position::new(20, 20), Size::new(10, 10)),
				ANIMATION_SECS,
			),
			lerp_anim(
				Transform::new(Position::new(30, 10), Size::new(20, 10)),
				ANIMATION_SECS,
			),
			lerp_anim(
				Transform::new(Position::new(0, 0), Size::new(50, 25)),
				ANIMATION_SECS,
			),
		]),
		true,
		pane(AnsiColor::Cyan),
	);

	let label = WidgetPointer::new(Label::new(
		"The quick brown fox jumps over the lazy dog.",
		LabelStyle::Single(Style::new().fg(Some(AnsiColor::Green))),
		false,
	));

	let label_ptr = label.pointer.clone();

	let root = Layers::new(vec![
		WidgetDyn::new(Box::new(root_1)),
		WidgetDyn::new(Box::new(root_2)),
		WidgetDyn::new(Box::new(root_3)),
		WidgetDyn::new(Box::new(label)),
	]);

	let mut context = Context::new(
		ContextConfig::default(),
		ContextSetupConfig {
			alt_screen: false,
			..Default::default()
		},
		XBackend::new().unwrap(),
		XContentProcessor {},
		root,
	);

	context.config.damaged_only = false;

	context.config.frame_delay =
		Some(std::time::Duration::from_secs_f64(FRAME_DELAY));

	context.setup().unwrap();

	let start = std::time::Instant::now();

	while std::time::Instant::now() - start
		< std::time::Duration::from_secs_f64(TIME_LIMIT)
	{
		let draw_summary = context.draw().unwrap().unwrap();

		label_ptr.lock().unwrap().set_label(format!(
			"Frame took {} seconds to draw.",
			draw_summary.duration.as_secs_f64(),
		));
	}

	context.cleanup().unwrap();
}
