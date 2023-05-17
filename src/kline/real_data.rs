use egui::{
    plot::{Bar, BoxElem, BoxSpread},
    Color32, Stroke,
};
use serde::{Deserialize, Serialize};

use super::utils::DateTimeUtils;

/// 这个类型是用来解析请求数据的。
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
    pub datetime: String,
}

/// k线图实际用到的数据
///
/// boxs是蜡烛图的数据
///
/// bars是成交量图的数据
///
/// datetime是获取到的时间字符串
#[derive(Debug, Clone)]
pub struct RealData {
    pub box_elem: BoxElem,
    pub bar: Bar,
    pub datetime: String,
}

impl RealData {
    /// 根据传入的candle和count，创建一个RealData类型的数据。
    /// 
    /// count会作为x轴坐标。
    pub fn new(candle: &Candle, count: f64) -> Self {
        let (quartile1, quartile3, color, bar_color) = if candle.open > candle.close {
            (candle.close, candle.open, Color32::GREEN, Color32::GREEN)
        } else if candle.open < candle.close {
            (candle.open, candle.close, Color32::RED, Color32::RED)
        } else {
            (candle.open, candle.close, Color32::BLACK, Color32::RED)
        };
        let median = (quartile1 + quartile3) / 2.0;
        let box_elem = BoxElem::new(
            count,
            BoxSpread::new(candle.low, quartile1, median, quartile3, candle.high),
        )
        .whisker_width(0.0)
        .fill(color)
        .stroke(Stroke::new(1.0, color));
        let bar = if candle.volume > 0.0 {
            Bar::new(count, candle.volume)
                .fill(bar_color)
                .stroke(Stroke::new(1.0, bar_color))
        } else {
            Bar::new(count, candle.volume)
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::new(0.0, Color32::TRANSPARENT))
        };
        Self {
            box_elem,
            bar,
            datetime: DateTimeUtils::format_datetime_string(&candle.datetime),
        }
    }
}
