use egui::{
    plot::{Bar, BarChart, BoxElem, BoxPlot, BoxSpread, HLine, Plot, PlotBounds, VLine},
    Color32, Context, PointerButton, Pos2, Response, Stroke, Ui, Vec2,
};
use serde::Serialize;
use web_sys::console;

use self::utils::{datetime_to_timestamp, timestamp_to_datetime, CustomError};

mod utils;

#[derive(Serialize)]
struct Candles {
    open: f64,
    close: f64,
    high: f64,
    low: f64,
    volume: f64,
    datetime: String,
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
    candles: Vec<Candles>,
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
    /// http是否已执行
    is_http_execute: bool,
}

impl Default for KLine {
    fn default() -> Self {
        Self {
            offset_pos: Pos2 { x: 0.0, y: 0.0 },
            size: Vec2 { x: 0.0, y: 0.0 },
            candles: vec![],
            has_new_data: true,
            x_range: AxisRange { min: 0.0, max: 0.0 },
            y_range: AxisRange {
                min: f64::INFINITY,
                max: f64::NEG_INFINITY,
            },
            y_volume_max: f64::NEG_INFINITY,
            drag_x_move: 0.0,
            half_distance: 30.0,
            is_candle_double_click: false,
            is_volume_double_click: false,
            is_http_execute: false,
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
    fn set_candles(&mut self) -> (Vec<BoxElem>, Vec<Bar>) {
        self.y_range_init();
        let boxs_bars = self
            .candles
            .iter()
            .map(|candle| {
                let x = datetime_to_timestamp(&candle.datetime);
                if x >= self.x_range.min && x <= self.x_range.max {
                    self.y_range.min = self.y_range.min.min(candle.low);
                    self.y_range.max = self.y_range.max.max(candle.high);
                    self.y_volume_max = self.y_volume_max.max(candle.volume);
                }
                let (quartile1, quartile3, color) = if candle.open > candle.close {
                    (candle.close, candle.open, Color32::GREEN)
                } else {
                    (candle.open, candle.close, Color32::RED)
                };
                let median = (quartile1 + quartile3) / 2.0;
                let box_elem = BoxElem::new(
                    x,
                    BoxSpread::new(candle.low, quartile1, median, quartile3, candle.high),
                )
                .whisker_width(0.0)
                .fill(color)
                .stroke(Stroke::new(1.0, color));
                let bar = Bar::new(x, candle.volume)
                    .fill(color)
                    .stroke(Stroke::new(1.0, color));
                (box_elem, bar)
            })
            .collect::<Vec<(BoxElem, Bar)>>();
        let boxs = boxs_bars
            .iter()
            .map(|item| item.0.to_owned())
            .collect::<Vec<BoxElem>>();
        let bars = boxs_bars
            .iter()
            .map(|item| item.1.to_owned())
            .collect::<Vec<Bar>>();
        (boxs, bars)
    }

    /// 初始化y_range
    fn y_range_init(&mut self) {
        self.y_range = AxisRange {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        };
        self.y_volume_max = f64::NEG_INFINITY;
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

    /// 创建k线图
    fn draw_kline(&mut self, ui: &mut Ui, candles: &Vec<BoxElem>, ctx: &Context) -> Response {
        Plot::new("kline")
            .width(self.size.x - 16.0)
            .height((self.size.y - 16.0) * 0.6)
            .allow_scroll(false)
            .allow_drag(false)
            .x_axis_formatter(|x, _r| {
                if x % 60.0 != 0.0 {
                    String::new()
                } else {
                    timestamp_to_datetime(x)
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
                // if self.is_candle_double_click {
                let bounds = plot_ui.plot_bounds();
                self.x_range = AxisRange {
                    min: bounds.min()[0],
                    max: bounds.max()[0],
                };
                // }
                self.add_space_y();
                let plot_bounds: PlotBounds = PlotBounds::from_min_max(
                    [self.x_range.min, self.y_range.min],
                    [self.x_range.max, self.y_range.max],
                );
                plot_ui.set_plot_bounds(plot_bounds);
                let box_plot = BoxPlot::new(candles.to_owned());
                plot_ui.box_plot(box_plot);

                if plot_ui.plot_hovered() {
                    if let Some(plot_point) = plot_ui.pointer_coordinate() {
                        plot_ui.hline(HLine::new(plot_point.y).color(Color32::BLACK));
                        plot_ui.vline(VLine::new(plot_point.x).color(Color32::BLACK));
                        if let Some(candle_boxelme) = candles.iter().find(|p| {
                            plot_point.x - self.half_distance < p.argument
                                && plot_point.x + self.half_distance > p.argument
                        }) {
                            if let Some(candle) = self.candles.iter().find(|p| {
                                datetime_to_timestamp(&p.datetime) == candle_boxelme.argument
                            }) {
                                egui::show_tooltip(ctx, egui::Id::new("tooltip"), |ui| {
                                    ui.label(format!("日期: {}", candle.datetime));
                                    ui.label(format!("开盘: {}", candle.open));
                                    ui.label(format!("最高: {}", candle.high));
                                    ui.label(format!("最低: {}", candle.low));
                                    ui.label(format!("收盘: {}", candle.close));
                                    ui.label(format!("数量: {}", candle.volume));
                                });
                            };
                        };
                    }
                }
            })
            .response
    }

    /// 创建成交量图
    fn draw_volume(
        &mut self,
        ui: &mut Ui,
        bars: &Vec<Bar>,
        candles: &Vec<BoxElem>,
        ctx: &Context,
    ) -> Response {
        Plot::new("kline_draw")
            .width(self.size.x - 16.0)
            .height((self.size.y - 16.0) * 0.4)
            .allow_scroll(false)
            .allow_drag(false)
            .x_axis_formatter(|x, _r| {
                if x % 60.0 != 0.0 {
                    String::new()
                } else {
                    timestamp_to_datetime(x)
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
                // let bounds = plot_ui.plot_bounds();
                // self.x_range = AxisRange {
                //     min: bounds.min()[0],
                //     max: bounds.max()[0],
                // };
                let plot_bounds: PlotBounds = PlotBounds::from_min_max(
                    [self.x_range.min, 0.0],
                    [self.x_range.max, self.y_volume_max],
                );
                plot_ui.set_plot_bounds(plot_bounds);
                let chart = BarChart::new(bars.to_owned());
                plot_ui.bar_chart(chart);

                if plot_ui.plot_hovered() {
                    if let Some(plot_point) = plot_ui.pointer_coordinate() {
                        plot_ui.hline(HLine::new(plot_point.y).color(Color32::BLACK));
                        plot_ui.vline(VLine::new(plot_point.x).color(Color32::BLACK));
                        if let Some(candle_boxelme) = candles.iter().find(|p| {
                            plot_point.x - self.half_distance < p.argument
                                && plot_point.x + self.half_distance > p.argument
                        }) {
                            if let Some(candle) = self.candles.iter().find(|p| {
                                datetime_to_timestamp(&p.datetime) == candle_boxelme.argument
                            }) {
                                egui::show_tooltip(ctx, egui::Id::new("tooltip"), |ui| {
                                    ui.label(format!("日期: {}", candle.datetime));
                                    ui.label(format!("开盘: {}", candle.open));
                                    ui.label(format!("最高: {}", candle.high));
                                    ui.label(format!("最低: {}", candle.low));
                                    ui.label(format!("收盘: {}", candle.close));
                                    ui.label(format!("数量: {}", candle.volume));
                                });
                            };
                        };
                    }
                }
            })
            .response
    }

    async fn http(&mut self) -> Result<String, CustomError> {
        let response = if self.is_http_execute {
            String::new()
        } else {
            let res = reqwest::get(
                "http://192.168.214.184:5100/optquote/getKLineData?code=CZCE.AP.AP401&ktype=m1",
            )
            .await?
            .text()
            .await?;
            self.is_http_execute = true;
            res
        };
        Ok(response)
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &Context) {
        self.set_size(ui);
        // let mut res = String::new();
        // async {
        //     res = self.http().await?;
        // }.await;
        // console::log_1(&format!("res: {}", res).into());
        let mut all_candles = vec![];
        self.candles = vec![
            Candles {
                open: 3306.32,
                close: 3206.32,
                high: 3444.32,
                low: 3103.32,
                volume: 2206.0,
                datetime: "2023-04-20 16:10".to_string(),
            },
            Candles {
                open: 3506.32,
                close: 3598.32,
                high: 3606.32,
                low: 3466.32,
                volume: 3206.0,
                datetime: "2023-04-20 16:11".to_string(),
            },
            Candles {
                open: 3306.32,
                close: 3406.32,
                high: 3456.32,
                low: 3206.32,
                volume: 1206.0,
                datetime: "2023-04-20 16:12".to_string(),
            },
            Candles {
                open: 2499.32,
                close: 2506.32,
                high: 2606.32,
                low: 2406.32,
                volume: 2606.0,
                datetime: "2023-04-20 16:13".to_string(),
            },
            Candles {
                open: 3106.32,
                close: 3306.32,
                high: 3406.32,
                low: 3006.32,
                volume: 3506.0,
                datetime: "2023-04-20 16:14".to_string(),
            },
        ];
        let (candles, bars) = self.set_candles();
        all_candles = candles;
        let candle_response = self.draw_kline(ui, &all_candles, ctx);
        let volume_response = self.draw_volume(ui, &bars, &all_candles, ctx);

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
    }
}
