use std::sync::mpsc::{self, Receiver};

use egui::{
    plot::{Bar, BarChart, BoxElem, BoxPlot, BoxSpread, HLine, Plot, PlotBounds, VLine},
    Color32, Context, Id, PointerButton, Pos2, Response, Stroke, Ui, Vec2,
};
use poll_promise::Promise;
use serde::{Deserialize, Serialize};
use web_sys::console;

use self::{
    real_data::{Candle, RealData},
    utils::{CustomError, DateTimeUtils},
};

mod real_data;
mod utils;

#[derive(Deserialize, Debug)]
struct CustomResponse {
    code: String,
    message: String,
    data: Vec<Candle>,
}

#[derive(Serialize)]
struct AxisRange {
    min: f64,
    max: f64,
}

#[derive(Serialize)]
pub struct KLine {
    /// k线图左上角的像素坐标
    offset_pos: Pos2,
    /// k线图的大小
    size: Vec2,
    /// 蜡烛图数据
    candles: Vec<Candle>,
    /// 蜡烛图的个数
    candles_count: f64,
    /// 是否有新数据
    has_new_data: bool,
    /// x轴的范围
    x_range: AxisRange,
    /// 蜡烛图y轴的范围
    y_range: AxisRange,
    /// 成交量图y轴的最大值
    y_volume_max: f64,
    /// x轴在drag状态下每帧的向量，正负表示方向
    drag_x_move: f32,
    /// 两个蜡烛图的x轴距离
    half_distance: f64,
    /// 当前帧是否双击蜡烛图
    is_candle_double_click: bool,
    /// 当前帧是否双击成交量图
    is_volume_double_click: bool,
    /// 十字线y轴的位置
    v_line_pos: f64,
    /// http是否已执行
    is_http_execute: bool,
    ///
    #[serde(skip)]
    promise: Option<Promise<CustomResponse>>,
}

impl Default for KLine {
    fn default() -> Self {
        Self {
            offset_pos: Pos2 { x: 0.0, y: 0.0 },
            size: Vec2 { x: 0.0, y: 0.0 },
            candles: vec![],
            candles_count: 1.0,
            has_new_data: true,
            x_range: AxisRange {
                min: -0.5,
                max: 0.0,
            },
            y_range: AxisRange {
                min: f64::INFINITY,
                max: f64::NEG_INFINITY,
            },
            y_volume_max: f64::NEG_INFINITY,
            drag_x_move: 0.0,
            half_distance: 0.3,
            is_candle_double_click: false,
            is_volume_double_click: false,
            v_line_pos: 0.0,
            is_http_execute: false,
            promise: Default::default(),
        }
    }
}

impl KLine {
    /// 设置k线图的size
    fn set_size(&mut self, ui: &Ui) {
        let clip_rect = ui.clip_rect();
        self.size = clip_rect.size();
    }

    /// 将新的k线数据转换为蜡烛图和成交量图的数据
    fn set_candles(&mut self) -> Vec<RealData> {
        let real_datas = self
            .candles
            .iter()
            .map(|candle| {
                let real_data = RealData::new(candle, self.candles_count);
                self.candles_count += 1.0;
                real_data
                // let x = DateTimeUtils::datetime_to_timestamp(&candle.datetime);
                // let (quartile1, quartile3, color) = if candle.open > candle.close {
                //     (candle.close, candle.open, Color32::GREEN)
                // } else {
                //     (candle.open, candle.close, Color32::RED)
                // };
                // let median = (quartile1 + quartile3) / 2.0;
                // let box_elem = BoxElem::new(
                //     self.candles_count,
                //     BoxSpread::new(candle.low, quartile1, median, quartile3, candle.high),
                // )
                // .whisker_width(0.0)
                // .fill(color)
                // .stroke(Stroke::new(1.0, color));
                // let bar = Bar::new(x, candle.volume)
                //     .fill(color)
                //     .stroke(Stroke::new(1.0, color));
                // (box_elem, bar)
            })
            .collect::<Vec<RealData>>();
        self.candles = vec![];
        real_datas
        // let boxs = boxs_bars
        //     .iter()
        //     .map(|item| item.0.to_owned())
        //     .collect::<Vec<BoxElem>>();
        // let bars = boxs_bars
        //     .iter()
        //     .map(|item| item.1.to_owned())
        //     .collect::<Vec<Bar>>();
        // (boxs, bars)
    }

    /// 初始化y_range
    fn y_range_init(&mut self) {
        self.y_range = AxisRange {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        };
        self.y_volume_max = f64::NEG_INFINITY;
    }

    /// 设置蜡烛图和成交量图的y轴范围。
    fn set_y_range(&mut self, real_datas: &Vec<RealData>) {
        self.y_range_init();
        real_datas.iter().for_each(|real_data| {
            if real_data.box_elem.argument >= self.x_range.min
                && real_data.box_elem.argument <= self.x_range.max
            {
                self.y_range.min = self.y_range.min.min(real_data.box_elem.spread.lower_whisker);
                self.y_range.max = self.y_range.max.max(real_data.box_elem.spread.upper_whisker);
                self.y_volume_max = self.y_volume_max.max(real_data.bar.value);
            }
        });
    }

    /// 增加y轴的范围，在上下边界产生一些空白
    fn add_space_y(&mut self) {
        if self.y_range.min != f64::INFINITY && self.y_range.max != f64::NEG_INFINITY {
            let space = (self.y_range.max - self.y_range.min) / 10.0;
            self.y_range = AxisRange {
                min: self.y_range.min - space,
                max: self.y_range.max + space,
            };
        }
        if self.y_volume_max != f64::NEG_INFINITY {
            self.y_volume_max *= 1.1;
        }
    }

    /// 设置x轴的范围
    // fn set_x_range(&mut self, min: f64, max: f64) {
    //     if min < -0.5 {
    //         self.x_range = AxisRange { min: -0.5, max };
    //     } else if max > self.candles_count {
    //         self.x_range = AxisRange {
    //             min,
    //             max: self.candles_count - 1.0,
    //         };
    //     } else {
    //         self.x_range = AxisRange { min, max };
    //     }
    // }

    /// 创建k线图
    fn draw_kline(&mut self, ui: &mut Ui, real_datas: &Vec<RealData>, ctx: &Context) -> Response {
        let datetimes = real_datas
            .iter()
            .map(|real_data| real_data.datetime.to_owned())
            .collect::<Vec<String>>();
        Plot::new("kline")
            .width(self.size.x - 16.0)
            .height((self.size.y - 16.0) * 0.6)
            .allow_scroll(false)
            .allow_drag(false)
            .x_axis_formatter(move |x, _r| {
                if x % 1.0 != 0.0 || datetimes.len() == 0 || x > datetimes.len() as f64 || x < 0.5 {
                    String::new()
                } else {
                    datetimes[(x - 1.0) as usize].to_owned()
                }
            })
            .show_y(false)
            .show_x(false)
            .label_formatter(|_name, _value| String::new())
            .show(ui, |plot_ui| {
                plot_ui.translate_bounds(Vec2 {
                    x: self.drag_x_move,
                    y: 0.0,
                });
                let bounds = plot_ui.plot_bounds();
                self.x_range = AxisRange {
                    min: bounds.min()[0],
                    max: bounds.max()[0],
                };
                self.add_space_y();
                let plot_bounds: PlotBounds = PlotBounds::from_min_max(
                    [self.x_range.min, self.y_range.min],
                    [self.x_range.max, self.y_range.max],
                );
                plot_ui.set_plot_bounds(plot_bounds);
                let box_plot = BoxPlot::new(
                    real_datas
                        .iter()
                        .map(|item| item.box_elem.to_owned())
                        .collect(),
                );
                plot_ui.box_plot(box_plot);

                // 使用K线图整体的y轴十字线
                plot_ui.vline(VLine::new(self.v_line_pos).color(Color32::BLACK));

                if plot_ui.plot_hovered() {
                    if let Some(plot_point) = plot_ui.pointer_coordinate() {
                        plot_ui.hline(HLine::new(plot_point.y).color(Color32::BLACK));
                        // 将位置信息赋值给v_line_pos以便全局使用。
                        self.v_line_pos = plot_point.x;
                        if let Some(real_data) = real_datas.iter().find(|real_data| {
                            plot_point.x - self.half_distance < real_data.box_elem.argument
                                && plot_point.x + self.half_distance > real_data.box_elem.argument
                        }) {
                            egui::show_tooltip(ctx, egui::Id::new("tooltip"), |ui| {
                                let (open, close) = if real_data.box_elem.fill == Color32::RED {
                                    (
                                        real_data.box_elem.spread.quartile1,
                                        real_data.box_elem.spread.quartile3,
                                    )
                                } else {
                                    (
                                        real_data.box_elem.spread.quartile3,
                                        real_data.box_elem.spread.quartile1,
                                    )
                                };
                                ui.label(format!("日期: {}", real_data.datetime));
                                ui.label(format!("开盘: {}", open));
                                ui.label(format!(
                                    "最高: {}",
                                    real_data.box_elem.spread.upper_whisker
                                ));
                                ui.label(format!(
                                    "最低: {}",
                                    real_data.box_elem.spread.lower_whisker
                                ));
                                ui.label(format!("收盘: {}", close));
                                ui.label(format!("数量: {}", real_data.bar.value));
                            });
                        };
                    }
                }
            })
            .response
    }

    /// 创建成交量图
    fn draw_volume(&mut self, ui: &mut Ui, real_datas: &Vec<RealData>, ctx: &Context) -> Response {
        Plot::new("kline_draw")
            .width(self.size.x - 16.0)
            .height((self.size.y - 16.0) * 0.4)
            .allow_scroll(false)
            .allow_drag(false)
            .x_axis_formatter(|_x, _r| String::new())
            .show_y(false)
            .show_x(false)
            .label_formatter(|_name, _value| String::new())
            .show(ui, |plot_ui| {
                plot_ui.translate_bounds(Vec2 {
                    x: self.drag_x_move,
                    y: 0.0,
                });
                let plot_bounds: PlotBounds = PlotBounds::from_min_max(
                    [self.x_range.min, 0.0],
                    [self.x_range.max, self.y_volume_max],
                );
                plot_ui.set_plot_bounds(plot_bounds);
                let chart = BarChart::new(
                    real_datas
                        .iter()
                        .map(|real_data| real_data.bar.to_owned())
                        .collect(),
                );
                plot_ui.bar_chart(chart);

                // 使用K线图整体的y轴十字线
                plot_ui.vline(VLine::new(self.v_line_pos).color(Color32::BLACK));

                if plot_ui.plot_hovered() {
                    if let Some(plot_point) = plot_ui.pointer_coordinate() {
                        plot_ui.hline(HLine::new(plot_point.y).color(Color32::BLACK));
                        // 将位置信息赋值给v_line_pos以便全局使用。
                        self.v_line_pos = plot_point.x;
                        if let Some(real_data) = real_datas.iter().find(|real_data| {
                            plot_point.x - self.half_distance < real_data.bar.argument
                                && plot_point.x + self.half_distance > real_data.bar.argument
                        }) {
                            egui::show_tooltip(ctx, egui::Id::new("tooltip"), |ui| {
                                let (open, close) = if real_data.box_elem.fill == Color32::RED {
                                    (
                                        real_data.box_elem.spread.quartile1,
                                        real_data.box_elem.spread.quartile3,
                                    )
                                } else {
                                    (
                                        real_data.box_elem.spread.quartile3,
                                        real_data.box_elem.spread.quartile1,
                                    )
                                };
                                ui.label(format!("日期: {}", real_data.datetime));
                                ui.label(format!("开盘: {}", open));
                                ui.label(format!(
                                    "最高: {}",
                                    real_data.box_elem.spread.upper_whisker
                                ));
                                ui.label(format!(
                                    "最低: {}",
                                    real_data.box_elem.spread.lower_whisker
                                ));
                                ui.label(format!("收盘: {}", close));
                                ui.label(format!("数量: {}", real_data.bar.value));
                            });
                        };
                    }
                }
            })
            .response
    }

    fn http(&mut self) {
        let (sender, promise) = Promise::new();
        wasm_bindgen_futures::spawn_local(async move {
            let client = reqwest::Client::new();
            let res = match client
                .get("http://localhost:8000/option/KlinePython/optquote/getKLineData")
                .query(&[("code", "CZCE.AP.AP401"), ("ktype", "m1")])
                // .fetch_mode_no_cors()
                .send()
                .await
            {
                Ok(response) => match response.json::<CustomResponse>().await {
                    Ok(custom_response) => custom_response,
                    Err(_) => CustomResponse {
                        code: "1".to_string(),
                        message: "解析错误".to_string(),
                        data: vec![],
                    },
                },
                Err(_) => CustomResponse {
                    code: "2".to_string(),
                    message: "请求错误".to_string(),
                    data: vec![],
                },
            };
            sender.send(res);
        });
        self.promise = Some(promise);
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &Context) {
        let mut saved_info = SaveInfo::load(ctx, Id::new("save_info")).unwrap_or_default();
        self.set_size(ui);
        if !self.is_http_execute {
            self.http();
            self.is_http_execute = true;
        }
        if let Some(promise) = &self.promise {
            if let Some(result) = promise.ready() {
                if result.code != "0" {
                    console::log_1(&format!("result: {:?}", result).into());
                } else {
                    self.candles = result.data.to_owned();
                }
                self.promise = None;
            } else {
                ui.ctx().request_repaint();
            }
        }
        let mut real_datas = self.set_candles();
        saved_info.real_datas.append(&mut real_datas);
        self.set_y_range(&saved_info.real_datas);
        let candle_response = self.draw_kline(ui, &saved_info.real_datas, ctx);
        let volume_response = self.draw_volume(ui, &saved_info.real_datas, ctx);

        // 拖动其中一个时，两个一起移动
        if candle_response.dragged_by(PointerButton::Primary)
            || volume_response.dragged_by(PointerButton::Primary)
        {
            self.drag_x_move = if candle_response.dragged() {
                -candle_response.drag_delta().x
            } else {
                -volume_response.drag_delta().x
            }
        } else {
            self.drag_x_move = 0.0;
        }

        let saving_info = SaveInfo {
            real_datas: saved_info.real_datas.to_owned(),
        };
        saving_info.store(ctx, Id::new("save_info"));
    }
}

/// 一些需要存储以供下一次渲染使用的数据
#[derive(Deserialize, Serialize, Clone, Debug)]
struct SaveInfo {
    #[serde(skip)]
    real_datas: Vec<RealData>,
}

impl Default for SaveInfo {
    fn default() -> Self {
        Self { real_datas: vec![] }
    }
}

impl SaveInfo {
    /// 存储数据
    pub fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut((|d| d.insert_persisted(id, self)));
    }

    /// 取出数据
    pub fn load(ctx: &Context, id: Id) -> Option<Self> {
        ctx.data_mut(|d| d.get_persisted(id))
    }
}
