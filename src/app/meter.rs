// This is taken from FasterThanLime's 'fam' project with metering.

use crate::egui::{pos2, vec2, Align2, Color32, FontId, Sense, Stroke, StrokeKind, Widget};
use std::time::{Duration, Instant};

pub struct Meter<'a> {
    values: &'a [f32],

    mapper: &'a dyn ValueMapper,

    ticks: &'a [Tick],

    sections: &'a [Section],

    bar_width: f32,

    show_max: bool,

    max_marker_duration: Duration,

    show_tick_values: bool,

    show_ticks_right: bool,

    text_above: Option<&'a str>,

    text_above_color: Option<Color32>,
}

pub struct Tick {
    value: f32,
    highlighted: bool,
}

impl Tick {
    pub const fn regular(value: f32) -> Self {
        Self {
            value,
            highlighted: false,
        }
    }

    pub const fn highlighted(value: f32) -> Self {
        Self {
            value,
            highlighted: true,
        }
    }
}

pub const DB_TICKS: [Tick; 8] = [
    Tick::regular(0.0),
    Tick::regular(-5.0),
    Tick::regular(-10.0),
    Tick::regular(-15.0),
    Tick::regular(-20.0),
    Tick::regular(-30.0),
    Tick::regular(-40.0),
    Tick::regular(-50.0),
];

pub struct Section {
    pub threshold: f32,
    pub color: Color32,
}

pub const DEFAULT_SECTIONS: [Section; 1] = [Section {
    threshold: f32::NEG_INFINITY,
    color: Color32::from_gray(170),
}];

pub const DB_GREEN_COLOR: Color32 = Color32::from_rgb(56, 201, 56);
pub const DB_YELLOW_COLOR: Color32 = Color32::from_rgb(242, 224, 26);
pub const DB_RED_COLOR: Color32 = Color32::from_rgb(225, 59, 29);

pub const DB_SECTIONS: [Section; 3] = [
    Section {
        threshold: f32::NEG_INFINITY,
        color: DB_GREEN_COLOR,
    },
    Section {
        threshold: -10.0,
        color: DB_YELLOW_COLOR,
    },
    Section {
        threshold: -5.0,
        color: DB_RED_COLOR,
    },
];

impl<'a> Meter<'a> {
    pub fn new(values: &'a [f32]) -> Self {
        Self {
            values,
            mapper: &(),
            ticks: &[],
            sections: &DEFAULT_SECTIONS,
            bar_width: 10.0,
            show_max: false,
            max_marker_duration: Duration::from_secs(5),
            show_tick_values: true,
            show_ticks_right: false,
            text_above: None,
            text_above_color: None,
        }
    }

    pub fn with_mapper(mut self, mapper: &'a dyn ValueMapper) -> Self {
        self.mapper = mapper;
        self
    }

    pub fn with_ticks(mut self, ticks: &'a [Tick]) -> Self {
        self.ticks = ticks;
        self
    }

    pub fn with_sections(mut self, sections: &'a [Section]) -> Self {
        self.sections = sections;
        self
    }

    pub fn with_bar_width(mut self, bar_width: f32) -> Self {
        self.bar_width = bar_width;
        self
    }

    pub fn show_max(mut self, show_max: bool) -> Self {
        self.show_max = show_max;
        self
    }

    pub fn show_tick_values(mut self, show_tick_values: bool) -> Self {
        self.show_tick_values = show_tick_values;
        self
    }

    pub fn show_ticks_right(mut self, show_ticks_right: bool) -> Self {
        self.show_ticks_right = show_ticks_right;
        self
    }

    pub fn with_text_above(mut self, text: &'a str) -> Self {
        self.text_above = Some(text);
        self
    }

    pub fn with_text_above_color(mut self, color: Option<Color32>) -> Self {
        self.text_above_color = color;
        self
    }
}

#[derive(Clone)]
struct MeterState {
    last_update: Instant,
    values: Vec<ValueState>,
}

impl MeterState {
    fn new(meter: &Meter) -> Self {
        let now = Instant::now();

        Self {
            last_update: now,
            values: meter
                .values
                .iter()
                .map(|&v| ValueState {
                    value: v,
                    max: v,
                    max_last_updated: now,
                })
                .collect(),
        }
    }

    fn update(&mut self, meter: &Meter) {
        let now = Instant::now();
        let dt = self.last_update.elapsed().as_secs_f32();
        self.last_update = now;

        if self.values.len() != meter.values.len() {
            *self = Self::new(meter);
        }

        for (vstate, &value) in self.values.iter_mut().zip(meter.values.iter()) {
            vstate.update(meter, value, dt);
        }
    }
}

#[derive(Clone)]
struct ValueState {
    value: f32,
    max: f32,
    max_last_updated: Instant,
}

impl ValueState {
    fn update(&mut self, meter: &Meter, value: f32, dt: f32) {
        let now = Instant::now();
        self.value = decay(self.value, dt);
        if value > self.value {
            self.value = value;
        }

        let max_duration_elapsed = self.max_last_updated.elapsed() > meter.max_marker_duration;

        if value > self.max || max_duration_elapsed {
            self.max = value;
            self.max_last_updated = now;
        }
    }
}

fn decay(x: f32, dt: f32) -> f32 {
    const TOTAL_DROP: f32 = 20.0;
    const FALL_TIME_SECONDS: f32 = 1.7;
    const DROP_PER_SECOND: f32 = TOTAL_DROP / FALL_TIME_SECONDS;

    x - (DROP_PER_SECOND * dt)
}

pub trait ValueMapper {
    fn to_unit_height(&self, x: f32) -> f32;
}

impl ValueMapper for () {
    fn to_unit_height(&self, x: f32) -> f32 {
        x
    }
}

pub struct DbMapper;

impl ValueMapper for DbMapper {
    fn to_unit_height(&self, value: f32) -> f32 {
        if value > -20.0 {
            // Map to [0, 1]
            let x = (value + 20.0) / 20.0;

            // Map to [0.5, 1]
            return x * 0.5 + 0.5;
        }

        let c: f32 = 0.08;
        (c * (value + 20.0)).exp() * 0.5
    }
}

impl Widget for Meter<'_> {
    fn ui(self, ui: &mut crate::egui::Ui) -> crate::egui::Response {
        let number_width: f32 = 15.0;
        let spacing: f32 = 4.0;
        let tick_width: f32 = 2.0;

        let mut req_width: f32 = 0.0;
        req_width += spacing;

        if self.show_tick_values {
            req_width += number_width;
            req_width += spacing;
        }

        req_width += tick_width;
        req_width += spacing;

        let bars_left = req_width;
        let bars_width = self.values.len() as f32 * self.bar_width;

        req_width += bars_width;

        if self.show_ticks_right {
            req_width += spacing;
            req_width += tick_width;
        }

        let (id, mut rect) = ui.allocate_space(vec2(req_width, ui.available_height()));
        let painter = ui.painter();

        let state: MeterState = ui.memory_mut(|mem| {
            let state = mem
                .data
                .get_temp_mut_or_insert_with(id, || MeterState::new(&self));
            state.update(&self);
            state.clone()
        });

        let top_space: f32 = 24.0;

        if let Some(text) = self.text_above {
            let color = self.text_above_color.unwrap_or(Color32::GRAY);

            painter.text(
                rect.min + vec2(bars_left + (bars_width / 2.0) - spacing, top_space - 8.0),
                Align2::CENTER_BOTTOM,
                text,
                FontId::proportional(10.0),
                color,
            );
        }

        rect.min.y += top_space;

        let tick_color = Color32::GRAY;
        let highlighted_tick_color = Color32::WHITE;

        let mut x = rect.min.x + spacing;
        if self.show_tick_values {
            for tick in self.ticks {
                let y = rect.bottom() - rect.height() * self.mapper.to_unit_height(tick.value);

                let color = if tick.highlighted {
                    highlighted_tick_color
                } else {
                    tick_color
                };

                painter.text(
                    pos2(rect.min.x + number_width, y),
                    Align2::RIGHT_CENTER,
                    format!("{}", tick.value),
                    FontId::proportional(9.0),
                    color,
                );
            }

            x += number_width + spacing;
        }

        let render_tick = |painter: &crate::egui::Painter, x: f32, tick: &Tick| {
            let y = rect.bottom() - rect.height() * self.mapper.to_unit_height(tick.value);

            let color = if tick.highlighted {
                highlighted_tick_color
            } else {
                tick_color
            };

            painter.line_segment(
                [pos2(x, y), pos2(x + tick_width, y)],
                Stroke::new(1.0, color),
            );
        };

        for tick in self.ticks {
            render_tick(painter, x, tick);
        }

        x += tick_width + spacing;

        for (i, vstate) in state.values.iter().enumerate() {
            let bar_rect = crate::egui::Rect::from_min_size(
                pos2(x + i as f32 * self.bar_width, rect.min.y),
                vec2(self.bar_width, rect.height()),
            );

            painter.rect_filled(bar_rect, 0.0, Color32::from_gray(20));

            let value_height = self.mapper.to_unit_height(vstate.value) * rect.height();

            for section in self.sections {
                let threshold_height =
                    self.mapper.to_unit_height(section.threshold) * rect.height();
                let section_height = (value_height - threshold_height).max(0.0);

                if section_height > 0.0 {
                    let section_rect = crate::egui::Rect::from_min_size(
                        pos2(
                            x + i as f32 * self.bar_width + 1.0,
                            rect.bottom() - threshold_height - section_height,
                        ),
                        vec2(self.bar_width - 2.0, section_height),
                    );
                    painter.rect_filled(section_rect, 0.0, section.color);
                }
            }

            painter.rect_stroke(
                bar_rect,
                0.0,
                Stroke::new(1.0, Color32::from_gray(10)),
                StrokeKind::Outside,
            );

            if self.show_max {
                let max_y = rect.bottom() - self.mapper.to_unit_height(vstate.max) * rect.height();

                let mut max_color = self.sections[0].color;
                for section in self.sections.iter().rev() {
                    if vstate.max >= section.threshold {
                        max_color = section.color;
                        break;
                    }
                }

                painter.line_segment(
                    [
                        pos2(bar_rect.min.x + 1.0, max_y),
                        pos2(bar_rect.min.x - 1.0, max_y),
                    ],
                    Stroke::new(1.0, max_color),
                );
            }
        }

        x += bars_width + spacing;

        if self.show_ticks_right {
            for tick in self.ticks {
                render_tick(painter, x, tick);
            }
        }

        ui.interact(rect, id, Sense::hover())
    }
}
