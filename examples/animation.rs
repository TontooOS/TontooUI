use tontooui::animation::{Easing, Repeat};
use tontooui::elements::{
    Align, Animated, AnimSpec, BasicText, BasicToolbar, Capsule, Circle,
    HStack, Keyframe, Phase, SFSymbolImage, Titlebar, TrafficAction, Transform,
    View, VStack,
};
use tontooui::renderer::FontSystem;
use tontooui::renderer::ImageLoader;
use tontooui::renderer::window::{App, Viewport, WindowCommand, run};
use tontooui::theme::{ThemeMode, ThemeWatcher};
use vello::Scene;
use vello::peniko::Color;

struct AnimationDemo {
    bar: Titlebar,
    stack: VStack,
    watcher: ThemeWatcher,
    focused: bool,
    bg: Color,
    command: Option<WindowCommand>,
}

impl AnimationDemo {
    fn new() -> Self {
        let slide = AnimSpec::new(0.9)
            .easing(Easing::CubicInOut)
            .autoreverse(true);
        // Fade plus slide-in on text, looping softly.
        let text = Animated::new(BasicText::new("Hello Tontoo"))
            .fade_in(slide)
            .move_from(0.0, 28.0, slide)
            .repeat(Repeat::Forever);
        // Endless spin on an SF Symbol (Spin child).
        let spin = Animated::new(
            SFSymbolImage::new("star.fill")
                .size(64.0)
                .color(Color::from_rgb8(0xff, 0x9f, 0x0a)),
        )
        .rotation(
            0.0,
            360.0,
            AnimSpec::new(2.5).easing(Easing::Linear),
        )
        .repeat(Repeat::Forever);
        // Pop scale on a capsule.
        let pop = Animated::new(Capsule::new(220.0, 80.0))
            .scale(
                0.6,
                1.0,
                AnimSpec::new(0.7)
                    .easing(Easing::BackOut)
                    .autoreverse(true),
            )
            .repeat(Repeat::Forever);
        // Keyframe bounce on a circle: up, down past rest, settle.
        let bounce = Animated::new(Circle::new(90.0).fill(Color::from_rgb8(
            0x34, 0xc7, 0x59,
        )))
        .keyframes(
            vec![
                Keyframe::at(0.0, Transform::identity()),
                Keyframe::new(
                    0.45,
                    Transform::offset(0.0, -48.0),
                    Easing::CubicOut,
                ),
                Keyframe::new(0.7, Transform::offset(0.0, 8.0), Easing::CubicIn),
                Keyframe::at(1.0, Transform::identity()),
            ],
            1.4,
        )
        .repeat(Repeat::Forever);
        // Phase patrol on a toolbar: right, left, back to rest.
        let patrol = Animated::new(BasicToolbar::new().icons(vec![
            "star.fill".to_string(),
            "heart.fill".to_string(),
            "bell.fill".to_string(),
        ]))
        .phases(
            vec![
                Phase::new(Transform::offset(60.0, 0.0), 0.6),
                Phase::new(Transform::offset(-60.0, 0.0), 0.6),
                Phase::new(Transform::identity(), 0.6),
            ],
            Easing::CubicInOut,
        )
        .repeat(Repeat::Forever);
        let stack = VStack::new()
            .align(Align::Center)
            .spacing(48.0)
            .child(text)
            .child(
                HStack::new()
                    .align(Align::Center)
                    .spacing(64.0)
                    .child(spin)
                    .child(pop),
            )
            .child(
                HStack::new()
                    .align(Align::Center)
                    .spacing(64.0)
                    .child(bounce)
                    .child(patrol),
            );
        Self {
            bar: Titlebar::new("Animation"),
            stack,
            watcher: ThemeWatcher::new(),
            focused: true,
            bg: tontooui::renderer::window::BACKGROUND,
            command: None,
        }
    }
}

impl App for AnimationDemo {
    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
        viewport: Viewport,
        time_secs: f64,
    ) {
        self.watcher.poll(time_secs);
        self.watcher.set_focused(self.focused, time_secs);
        let palette = self.watcher.palette(time_secs);
        self.bg = palette.bg;
        let theme = self.watcher.theme();
        let dark = theme.mode == ThemeMode::Dark;
        let (mode, glass, focused) = (theme.mode, theme.glass, self.focused);
        if let Some(anim) = self.stack.child_mut::<Animated<BasicText>>(0) {
            anim.child_mut().set_theme(mode);
            anim.child_mut().set_focused(focused);
        }
        // Row stacks hold the remaining animated elements.
        for row in 1..self.stack.len() {
            if let Some(hstack) = self.stack.child_mut::<HStack>(row) {
                for index in 0..hstack.len() {
                    if let Some(anim) =
                        hstack.child_mut::<Animated<SFSymbolImage>>(index)
                    {
                        anim.child_mut().set_theme(palette.text, dark);
                        anim.child_mut().set_focused(focused);
                    } else if let Some(anim) =
                        hstack.child_mut::<Animated<Capsule>>(index)
                    {
                        anim.child_mut().set_focused(focused);
                    } else if let Some(anim) =
                        hstack.child_mut::<Animated<Circle>>(index)
                    {
                        anim.child_mut().set_focused(focused);
                    } else if let Some(anim) =
                        hstack.child_mut::<Animated<BasicToolbar>>(index)
                    {
                        anim.child_mut().set_theme(mode, glass);
                        anim.child_mut().set_focused(focused);
                    }
                }
            }
        }

        self.bar.set_palette(
            palette.titlebar_bg,
            palette.titlebar_text,
            palette.divider,
        );
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);

        let top = viewport.y + 31.0;
        let (stack_w, stack_h) = self.stack.measure(fonts);
        let x = viewport.x + ((viewport.width - stack_w) / 2.0).max(0.0);
        self.stack
            .place(fonts, x, top + 40.0, stack_w, stack_h);
        self.stack.draw(scene, fonts, images);
    }

    fn background(&self) -> Color {
        self.bg
    }

    fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
        Some(self.bar.drag_rect())
    }

    fn poll_window_command(&mut self) -> Option<WindowCommand> {
        self.command.take()
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        match self.bar.press(x as f32, y as f32) {
            Some(TrafficAction::Close) => self.command = Some(WindowCommand::Close),
            Some(TrafficAction::Minimize) => {
                self.command = Some(WindowCommand::Minimize)
            }
            Some(TrafficAction::Maximize) => {
                self.command = Some(WindowCommand::ToggleMaximize)
            }
            None => {}
        }
    }

    fn mouse_move(&mut self, x: f64, y: f64) {
        self.bar.set_hover(x as f32, y as f32);
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.bar.set_focused(focused);
    }
}

fn main() {
    if let Err(err) = run("Animation", 900, 900, AnimationDemo::new()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
