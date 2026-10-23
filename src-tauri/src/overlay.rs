//! Pure preview geometry and lifecycle policy. No desktop APIs in this module.
use crate::targets::Identity;
use serde::{Deserialize, Serialize};

pub type Binding = (Identity, u64);

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Position {
    pub center: bool,
    pub x: Option<f64>,
    pub y: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Geometry {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
    pub dpi: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub geometry: Geometry,
    pub point: Option<(i32, i32)>,
    pub label: String,
}

pub fn frame(geometry: Geometry, position: Position) -> Option<Frame> {
    let Geometry {
        left,
        top,
        width,
        height,
        ..
    } = geometry;
    if width <= 0
        || height <= 0
        || left.checked_add(width).is_none()
        || top.checked_add(height).is_none()
    {
        return None;
    }
    let point = if position.center {
        Some((width / 2, height / 2))
    } else {
        position
            .x
            .zip(position.y)
            .filter(|(x, y)| {
                x.is_finite()
                    && y.is_finite()
                    && x.fract() == 0.0
                    && y.fract() == 0.0
                    && *x >= 0.0
                    && *y >= 0.0
                    && *x < width as f64
                    && *y < height as f64
            })
            .map(|(x, y)| (x as i32, y as i32))
    };
    let label = match point {
        Some((x, y)) => format!("{width} x {height} | X {x}  Y {y}"),
        None => format!("{width} x {height} | Invalid X/Y"),
    };
    Some(Frame {
        geometry,
        point,
        label,
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub revision: u64,
    pub enabled: bool,
    pub visible: bool,
    pub message: String,
}
impl Default for Status {
    fn default() -> Self {
        Self {
            revision: 0,
            enabled: false,
            visible: false,
            message: "坐标预览已关闭。".into(),
        }
    }
}

pub struct Observation {
    pub binding: Option<Binding>,
    pub foreground: bool,
    pub visible: bool,
    pub minimized: bool,
    pub cloaked: bool,
    pub geometry: Option<Geometry>,
}

pub trait Renderer {
    fn show(&mut self, frame: &Frame) -> Result<bool, ()>;
    fn hide(&mut self) -> Result<(), ()>;
    fn close(&mut self);
}

#[derive(Default)]
pub struct Engine {
    request: Option<(Binding, Position)>,
    pub status: Status,
}
impl Engine {
    fn status(&mut self, enabled: bool, visible: bool, message: &str) {
        if (
            self.status.enabled,
            self.status.visible,
            self.status.message.as_str(),
        ) != (enabled, visible, message)
        {
            self.status.revision += 1;
            self.status.enabled = enabled;
            self.status.visible = visible;
            self.status.message = message.into();
        }
    }
    pub fn configure(&mut self, binding: Binding, position: Position) {
        self.request = Some((binding, position));
        self.status(
            true,
            false,
            "等待目标成为前台窗口后显示；不会自动激活目标。明示位置为当前客户区像素。",
        );
    }
    pub fn requested_binding(&self) -> Option<&Binding> {
        self.request.as_ref().map(|(binding, _)| binding)
    }
    pub fn stop(&mut self, renderer: &mut impl Renderer, message: &str) {
        self.request = None;
        renderer.close();
        self.status(false, false, message);
    }
    pub fn tick(&mut self, observation: Observation, renderer: &mut impl Renderer) {
        let Some((binding, position)) = &self.request else {
            return;
        };
        if observation.binding.as_ref() != Some(binding) {
            self.stop(renderer, "目标已解绑、换绑或失效，坐标预览已关闭。");
            return;
        }
        if !observation.foreground
            || !observation.visible
            || observation.minimized
            || observation.cloaked
        {
            if renderer.hide().is_err() {
                self.fail(renderer);
            } else {
                self.status(
                    true,
                    false,
                    "预览已暂停：目标需为前台、可见且非最小化窗口。返回目标后自动恢复。",
                );
            }
            return;
        }
        let Some(frame) = observation
            .geometry
            .and_then(|geometry| frame(geometry, *position))
        else {
            if renderer.hide().is_err() {
                self.fail(renderer);
            } else {
                self.status(
                    true,
                    false,
                    "预览已暂停：无法读取有效客户区，返回有效窗口后恢复。",
                );
            }
            return;
        };
        match renderer.show(&frame) {
            Err(()) => self.fail(renderer),
            Ok(false) => self.status(
                true,
                false,
                "预览已暂停：目标需为前台、可见且非最小化窗口。返回目标后自动恢复。",
            ),
            Ok(true) => self.status(
                true,
                true,
                if frame.point.is_some() {
                    "预览显示中；只展示坐标，不发送输入。"
                } else {
                    "坐标无效：请填写客户区内的非负整数 X/Y；当前不显示点击点。"
                },
            ),
        }
    }
    fn fail(&mut self, renderer: &mut impl Renderer) {
        self.stop(
            renderer,
            "原生坐标预览创建或更新失败，已关闭；可重试，不影响主界面。",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn binding(epoch: u64) -> Binding {
        (
            Identity {
                hwnd: 1,
                pid: 2,
                created: 3,
                thread: 4,
            },
            epoch,
        )
    }
    fn position() -> Position {
        Position {
            center: true,
            x: None,
            y: None,
        }
    }
    fn geometry() -> Geometry {
        Geometry {
            left: -1920,
            top: -200,
            width: 1001,
            height: 501,
            dpi: 144,
        }
    }
    fn observation() -> Observation {
        Observation {
            binding: Some(binding(1)),
            foreground: true,
            visible: true,
            minimized: false,
            cloaked: false,
            geometry: Some(geometry()),
        }
    }
    #[derive(Default)]
    struct Fake {
        visible: bool,
        closed: usize,
        frames: Vec<Frame>,
        fail: bool,
    }
    impl Renderer for Fake {
        fn show(&mut self, frame: &Frame) -> Result<bool, ()> {
            if self.fail {
                return Err(());
            }
            self.visible = true;
            self.frames.push(frame.clone());
            Ok(true)
        }
        fn hide(&mut self) -> Result<(), ()> {
            if self.fail {
                return Err(());
            }
            self.visible = false;
            Ok(())
        }
        fn close(&mut self) {
            self.visible = false;
            self.closed += 1;
        }
    }
    #[test]
    fn center_custom_edges_and_invalid_coordinates() {
        assert_eq!(
            frame(geometry(), position()).unwrap().point,
            Some((500, 250))
        );
        for (x, y, expected) in [
            (0.0, 0.0, Some((0, 0))),
            (1000.0, 500.0, Some((1000, 500))),
            (1001.0, 0.0, None),
            (-1.0, 2.0, None),
            (2.5, 1.0, None),
            (f64::NAN, 1.0, None),
            (f64::INFINITY, 0.0, None),
        ] {
            assert_eq!(
                frame(
                    geometry(),
                    Position {
                        center: false,
                        x: Some(x),
                        y: Some(y)
                    }
                )
                .unwrap()
                .point,
                expected
            );
        }
        assert!(frame(
            geometry(),
            Position {
                center: false,
                x: None,
                y: Some(1.0)
            }
        )
        .unwrap()
        .point
        .is_none());
        assert!(frame(
            Geometry {
                width: 0,
                ..geometry()
            },
            position()
        )
        .is_none());
        assert!(frame(
            Geometry {
                left: i32::MAX,
                ..geometry()
            },
            position()
        )
        .is_none());
    }
    #[test]
    fn negative_monitor_origin_and_dpi_do_not_scale_client_pixels_twice() {
        let a = frame(geometry(), position()).unwrap();
        assert_eq!(
            (
                a.geometry.left + a.point.unwrap().0,
                a.geometry.top + a.point.unwrap().1
            ),
            (-1420, 50)
        );
        for dpi in [96, 120, 144, 192] {
            let b = frame(
                Geometry {
                    dpi,
                    left: -2500,
                    ..geometry()
                },
                position(),
            )
            .unwrap();
            assert_eq!(a.point, b.point);
        }
        let custom = Position {
            center: false,
            x: Some(300.0),
            y: Some(100.0),
        };
        assert_eq!(
            frame(
                Geometry {
                    width: 2000,
                    height: 1000,
                    ..geometry()
                },
                custom
            )
            .unwrap()
            .point,
            Some((300, 100))
        );
        assert_eq!(
            frame(
                Geometry {
                    width: 2000,
                    height: 1000,
                    ..geometry()
                },
                position()
            )
            .unwrap()
            .point,
            Some((1000, 500))
        );
    }
    #[test]
    fn foreground_visibility_pause_resume_and_invalid_geometry() {
        let mut engine = Engine::default();
        let mut renderer = Fake::default();
        assert!(!engine.status.enabled);
        engine.configure(binding(1), position());
        engine.tick(observation(), &mut renderer);
        assert!(renderer.visible);
        for cause in 0..5 {
            let mut paused = observation();
            match cause {
                0 => paused.foreground = false,
                1 => paused.visible = false,
                2 => paused.minimized = true,
                3 => paused.cloaked = true,
                _ => paused.geometry = None,
            };
            engine.tick(paused, &mut renderer);
            assert!(!renderer.visible);
            assert!(engine.status.enabled);
            engine.tick(observation(), &mut renderer);
            assert!(renderer.visible);
        }
    }
    #[test]
    fn binding_epoch_or_identity_change_exit_and_renderer_failures_close() {
        for cause in 0..5 {
            let mut engine = Engine::default();
            let mut renderer = Fake::default();
            engine.configure(binding(1), position());
            engine.tick(observation(), &mut renderer);
            let mut changed = observation();
            match cause {
                0 => changed.binding = None,
                1 => changed.binding = Some(binding(2)),
                2 => changed.binding.as_mut().unwrap().0.created += 1,
                3 => renderer.fail = true,
                _ => {
                    renderer.fail = true;
                    changed.foreground = false;
                }
            };
            engine.tick(changed, &mut renderer);
            assert!(!engine.status.enabled);
            assert!(!renderer.visible);
            assert_eq!(renderer.closed, 1);
            engine.tick(observation(), &mut renderer);
            assert!(!renderer.visible);
        }
        let mut engine = Engine::default();
        let mut renderer = Fake::default();
        engine.configure(binding(1), position());
        engine.stop(&mut renderer, "off");
        assert!(!engine.status.enabled);
        assert_eq!(renderer.closed, 1);
    }
}
