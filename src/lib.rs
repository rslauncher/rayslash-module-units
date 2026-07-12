#[allow(warnings)]
mod bindings;

use bindings::exports::rayslash::module::provider::Guest;
use bindings::rayslash::module::types::{
    Action, Icon, ModuleError, QueryContext, QueryResponse, ResultItem,
};

struct Component;

impl Guest for Component {
    fn query(context: QueryContext) -> Result<QueryResponse, ModuleError> {
        let Some(conversion) = convert(&context.query) else {
            return Ok(QueryResponse {
                results: Vec::new(),
                exclusive: false,
            });
        };
        let (expression, result) = match conversion {
            Ok(value) => value,
            Err(message) => {
                return Ok(QueryResponse {
                    results: vec![ResultItem {
                        id: format!("units:error:{}", context.query.trim().to_ascii_lowercase()),
                        title: message.clone(),
                        subtitle: format!("Convert: {}", context.query.trim()),
                        icon: Icon::Text("U".into()),
                        score: None,
                        action: Action::ShowMessage(message),
                    }],
                    exclusive: true,
                });
            }
        };
        Ok(QueryResponse {
            results: vec![ResultItem {
                id: format!("units:{}", expression.to_ascii_lowercase()),
                title: result.clone(),
                subtitle: format!("Convert: {expression}"),
                icon: Icon::Text("U".into()),
                score: None,
                action: Action::CopyText(result),
            }],
            exclusive: true,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Dimension {
    Length,
    Mass,
    Volume,
    Temperature,
}

#[derive(Clone, Copy)]
struct Unit {
    symbol: &'static str,
    dimension: Dimension,
    factor: f64,
}

fn convert(input: &str) -> Option<Result<(String, String), String>> {
    let words = input.split_whitespace().collect::<Vec<_>>();
    if words.len() != 4 || !matches!(words[2].to_ascii_lowercase().as_str(), "to" | "in") {
        return None;
    }
    let amount = match words[0].replace(',', "").parse::<f64>() {
        Ok(value) if value.is_finite() => value,
        _ => return None,
    };
    let from = unit(words[1])?;
    let Some(to) = unit(words[3]) else {
        return Some(Err(format!("Unsupported unit: {}", words[3])));
    };
    if from.dimension != to.dimension {
        return Some(Err("Units belong to different dimensions.".into()));
    }
    let converted = if from.dimension == Dimension::Temperature {
        let celsius = match from.symbol {
            "°C" => amount,
            "°F" => (amount - 32.0) * 5.0 / 9.0,
            "K" => amount - 273.15,
            _ => unreachable!(),
        };
        match to.symbol {
            "°C" => celsius,
            "°F" => celsius * 9.0 / 5.0 + 32.0,
            "K" => celsius + 273.15,
            _ => unreachable!(),
        }
    } else {
        amount * from.factor / to.factor
    };
    let expression = format!("{} {} to {}", format_number(amount), from.symbol, to.symbol);
    Some(Ok((
        expression,
        format!("{} {}", format_number(converted), to.symbol),
    )))
}

fn unit(value: &str) -> Option<Unit> {
    let value = value.trim().to_ascii_lowercase();
    let unit = match value.as_str() {
        "m" | "meter" | "meters" | "metre" | "metres" => ("m", Dimension::Length, 1.0),
        "km" | "kilometer" | "kilometers" | "kilometre" | "kilometres" => {
            ("km", Dimension::Length, 1000.0)
        }
        "cm" | "centimeter" | "centimeters" | "centimetre" | "centimetres" => {
            ("cm", Dimension::Length, 0.01)
        }
        "mm" | "millimeter" | "millimeters" => ("mm", Dimension::Length, 0.001),
        "in" | "inch" | "inches" => ("in", Dimension::Length, 0.0254),
        "ft" | "foot" | "feet" => ("ft", Dimension::Length, 0.3048),
        "yd" | "yard" | "yards" => ("yd", Dimension::Length, 0.9144),
        "mi" | "mile" | "miles" => ("mi", Dimension::Length, 1609.344),
        "kg" | "kilogram" | "kilograms" => ("kg", Dimension::Mass, 1.0),
        "g" | "gram" | "grams" => ("g", Dimension::Mass, 0.001),
        "mg" | "milligram" | "milligrams" => ("mg", Dimension::Mass, 0.000001),
        "lb" | "lbs" | "pound" | "pounds" => ("lb", Dimension::Mass, 0.45359237),
        "oz" | "ounce" | "ounces" => ("oz", Dimension::Mass, 0.028349523125),
        "l" | "liter" | "liters" | "litre" | "litres" => ("L", Dimension::Volume, 1.0),
        "ml" | "milliliter" | "milliliters" => ("mL", Dimension::Volume, 0.001),
        "gal" | "gallon" | "gallons" => ("gal", Dimension::Volume, 3.785411784),
        "qt" | "quart" | "quarts" => ("qt", Dimension::Volume, 0.946352946),
        "cup" | "cups" => ("cup", Dimension::Volume, 0.2365882365),
        "c" | "°c" | "celsius" => ("°C", Dimension::Temperature, 1.0),
        "f" | "°f" | "fahrenheit" => ("°F", Dimension::Temperature, 1.0),
        "k" | "kelvin" => ("K", Dimension::Temperature, 1.0),
        _ => return None,
    };
    Some(Unit {
        symbol: unit.0,
        dimension: unit.1,
        factor: unit.2,
    })
}

fn format_number(value: f64) -> String {
    if (value - value.round()).abs() < 1e-10 {
        return format!("{:.0}", value);
    }
    let text = format!("{value:.8}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

bindings::export!(Component with_types_in bindings);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn converts_length() {
        assert_eq!(convert("10 km to mi").unwrap().unwrap().1, "6.21371192 mi");
    }
    #[test]
    fn converts_temperature() {
        assert_eq!(convert("32 f to c").unwrap().unwrap().1, "0 °C");
    }
    #[test]
    fn rejects_mixed_dimensions() {
        assert!(convert("1 kg to m").unwrap().is_err());
    }
}
