use backend::XBackend;
use tuigui::*;

mod backend;

// Amount of time before the demo quits.
const TIME_LIMIT: f64 = 6.0;

// Play with this to see that a lower FPS doesn't affect the
// animation's speed. (It simply just makes it choppy.)
const FRAME_DELAY: f64 = 0.0;

// The amount of seconds the animations in this demo will take to finish.
const ANIMATION_SECS: f64 = 1.0;

fn pane(color: AnsiColor) -> widgets::FillArea {
	widgets::FillArea::new(Some(Content::Styled(
		'█',
		Style::new().fg(Some(color)),
	)))
}

fn main() {
	// Play with me to change what custom lerp animation gets used.
	let lerp_anim = animations::linear;

	let root_1 = widgets::AnimationContainer::new(
		widgets::AnimationContainerSequence::Many(vec![
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

	let root_2 = widgets::AnimationContainer::new(
		widgets::AnimationContainerSequence::Single(lerp_anim(
			Transform::new(Position::new(60, 10), Size::new(30, 10)),
			ANIMATION_SECS,
		)),
		true,
		pane(AnsiColor::Red),
	);

	let root_3 = widgets::AnimationContainer::new(
		widgets::AnimationContainerSequence::Many(vec![
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

	let root = widgets::Layers::new(vec![root_1, root_2, root_3]);

	let mut context = Context::new(
		ContextConfig::default(),
		ContextSetupConfig {
			alt_screen: false,
			..Default::default()
		},
		XBackend::new().unwrap(),
		content_processors::OwoColorsContentProcessor::new(),
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
		context.draw().unwrap();
	}

	context.cleanup().unwrap();
}