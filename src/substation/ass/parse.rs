use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take_until},
    character::complete::{char, i64, space0},
    combinator::{fail, map, opt, rest},
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult, Parser,
};

use crate::{
    substation::{
        ass::{AssEvent, AssScriptInfo, AssStyle},
        common::{
            data::SubStationEventKind,
            parse::{
                parse_category_header, parse_fonts, parse_graphics, parse_reverse_bool,
                parse_timestamp, SubStationSection,
            },
        },
    },
    AssSubtitle, Moment,
};

#[derive(Debug)]
enum AssCategory<'a> {
    ScriptInfo((&'a str, &'a str)),
    Styles(AssStyle),
    Events(AssEvent),
    Fonts(&'a str),
    Graphics(&'a str),
}

type EventTuple = (
    i64,
    Moment,
    Moment,
    Option<String>,
    Option<String>,
    i64,
    i64,
    i64,
    Option<String>,
    String,
);

fn parse_event_line(input: &str) -> IResult<&str, EventTuple> {
    preceded(
        tuple((space0, char(':'), space0)),
        tuple((
            terminated(i64, pair(char(','), space0)),
            terminated(parse_timestamp, pair(char(','), space0)),
            terminated(parse_timestamp, pair(char(','), space0)),
            terminated(
                map(opt(take_until(",")), |s| s.map(|v: &str| v.to_string())),
                pair(char(','), space0),
            ),
            terminated(
                map(opt(take_until(",")), |s| s.map(|v: &str| v.to_string())),
                pair(char(','), space0),
            ),
            terminated(i64, pair(char(','), space0)),
            terminated(i64, pair(char(','), space0)),
            terminated(i64, pair(char(','), space0)),
            terminated(
                map(opt(take_until(",")), |s| s.map(|v: &str| v.to_string())),
                pair(char(','), space0),
            ),
            map(rest, |s: &str| s.to_string()),
        )),
    )
    .parse(input)
}

fn map_ass_event<I, E>(
    parser: impl Parser<I, EventTuple, E>,
    kind: SubStationEventKind,
) -> impl FnMut(I) -> IResult<I, AssEvent, E> {
    map(
        parser,
        move |(layer, start, end, style, name, margin_l, margin_r, margin_v, effect, text)| {
            AssEvent {
                kind,
                layer,
                start,
                end,
                style,
                name,
                margin_l,
                margin_r,
                margin_v,
                effect,
                text,
            }
        },
    )
}

fn parse_dialogue(input: &str) -> IResult<&str, AssEvent> {
    map_ass_event(
        preceded(tag_no_case("Dialogue"), parse_event_line),
        SubStationEventKind::Dialogue,
    )
    .parse(input)
}

fn parse_picture(input: &str) -> IResult<&str, AssEvent> {
    map_ass_event(
        preceded(tag_no_case("Picture"), parse_event_line),
        SubStationEventKind::Picture,
    )
    .parse(input)
}

fn parse_sound(input: &str) -> IResult<&str, AssEvent> {
    map_ass_event(
        preceded(tag_no_case("Sound"), parse_event_line),
        SubStationEventKind::Sound,
    )
    .parse(input)
}

fn parse_movie(input: &str) -> IResult<&str, AssEvent> {
    map_ass_event(
        preceded(tag_no_case("Movie"), parse_event_line),
        SubStationEventKind::Movie,
    )
    .parse(input)
}

fn parse_command(input: &str) -> IResult<&str, AssEvent> {
    map_ass_event(
        preceded(tag_no_case("Command"), parse_event_line),
        SubStationEventKind::Command,
    )
    .parse(input)
}

fn parse_event(input: &str) -> IResult<&str, AssCategory> {
    map(
        alt((
            parse_dialogue,
            parse_picture,
            parse_sound,
            parse_movie,
            parse_command,
        )),
        AssCategory::Events,
    )
    .parse(input)
}

fn parse_style_line(input: &str) -> IResult<&str, AssCategory> {
    map(
        preceded(
            tuple((tag_no_case("Style"), space0, char(':'), space0)),
            pair(
                tuple((
                    terminated(take_until(","), pair(char(','), space0)), // Name
                    terminated(take_until(","), pair(char(','), space0)), // Fontname
                    terminated(i64, pair(char(','), space0)),             // Fontsize
                    terminated(take_until(","), pair(char(','), space0)), // PrimaryColour
                    terminated(take_until(","), pair(char(','), space0)), // SecondaryColour
                    terminated(take_until(","), pair(char(','), space0)), // OutlineColour
                    terminated(take_until(","), pair(char(','), space0)), // BackColour
                    terminated(parse_reverse_bool, pair(char(','), space0)), // Bold
                    terminated(parse_reverse_bool, pair(char(','), space0)), // Italic
                    terminated(parse_reverse_bool, pair(char(','), space0)), // Underline
                    terminated(parse_reverse_bool, pair(char(','), space0)), // StrikeOut
                )),
                tuple((
                    terminated(i64, pair(char(','), space0)),    // ScaleX
                    terminated(i64, pair(char(','), space0)),    // ScaleY
                    terminated(double, pair(char(','), space0)), // Spacing
                    terminated(double, pair(char(','), space0)), // Angle
                    terminated(i64, pair(char(','), space0)),    // BorderStyle
                    terminated(double, pair(char(','), space0)), // Outline
                    terminated(double, pair(char(','), space0)), // Shadow
                    terminated(i64, pair(char(','), space0)),    // Alignment
                    terminated(i64, pair(char(','), space0)),    // MarginL
                    terminated(i64, pair(char(','), space0)),    // MarginR
                    terminated(i64, pair(char(','), space0)),    // MarginV
                    i64,                                         // Encoding
                )),
            ),
        ),
        |(
            (
                name,
                fontname,
                fontsize,
                primary_colour,
                secondary_colour,
                outline_colour,
                back_colour,
                bold,
                italic,
                underline,
                strike_out,
            ),
            (
                scale_x,
                scale_y,
                spacing,
                angle,
                border_style,
                outline,
                shadow,
                alignment,
                margin_l,
                margin_r,
                margin_v,
                encoding,
            ),
        )| {
            AssCategory::Styles(AssStyle {
                name: name.to_string(),
                fontname: fontname.to_string(),
                fontsize,
                primary_colour: primary_colour.to_string(),
                secondary_colour: secondary_colour.to_string(),
                outline_colour: outline_colour.to_string(),
                back_colour: back_colour.to_string(),
                bold,
                italic,
                underline,
                strike_out,
                scale_x,
                scale_y,
                spacing,
                angle,
                border_style,
                outline,
                shadow,
                alignment,
                margin_l,
                margin_r,
                margin_v,
                encoding,
            })
        },
    )
    .parse(input)
}

fn parse_script_info_line(input: &str) -> IResult<&str, AssCategory> {
    map(
        separated_pair(take_until(":"), delimited(space0, char(':'), space0), rest),
        AssCategory::ScriptInfo,
    )
    .parse(input)
}

fn parse_font_line(input: &str) -> IResult<&str, AssCategory> {
    map(rest, AssCategory::Fonts).parse(input)
}

fn parse_graphic_line(input: &str) -> IResult<&str, AssCategory> {
    map(rest, AssCategory::Graphics).parse(input)
}

fn parse_nothing(input: &str) -> IResult<&str, AssCategory> {
    fail(input)
}

fn build_script_info(data: &HashMap<String, String>) -> AssScriptInfo {
    AssScriptInfo {
        title: data.get("Title").cloned(),
        original_script: data.get("Original Script").cloned(),
        original_translation: data.get("Original Translation").cloned(),
        original_editing: data.get("Original Editing").cloned(),
        original_timing: data.get("Original Timing").cloned(),
        synch_point: data.get("Synch Point").cloned(),
        script_updated_by: data.get("Script Updated By").cloned(),
        update_details: data.get("Update Details").cloned(),
        script_type: data.get("ScriptType").cloned(),
        collisions: data.get("Collisions").cloned(),
        play_res_y: data.get("PlayResY").cloned(),
        play_res_x: data.get("PlayResX").cloned(),
        play_depth: data.get("PlayDepth").cloned(),
        timer: data.get("Timer").cloned(),
        wrap_style: data.get("WrapStyle").cloned(),
    }
}

pub(crate) fn parse_ass<T: Read>(reader: BufReader<T>) -> AssSubtitle {
    let mut raw_script_info = HashMap::new();
    let mut dialogue = Vec::new();
    let mut pictures = Vec::new();
    let mut sounds = Vec::new();
    let mut movies = Vec::new();
    let mut commands = Vec::new();
    let mut styles = Vec::new();
    let mut raw_graphics = Vec::new();
    let mut raw_fonts = Vec::new();
    let mut state = None;
    for line in reader.lines() {
        let Ok(line) = line else {
            continue;
        };
        if let Ok((_, category)) = parse_category_header(line.as_str()) {
            state = Some(category);
            continue;
        }

        let parse_fn = match state {
            Some(SubStationSection::Events) => parse_event,
            Some(SubStationSection::Styles) => parse_style_line,
            Some(SubStationSection::ScriptInfo) => parse_script_info_line,
            Some(SubStationSection::Graphics) => parse_graphic_line,
            Some(SubStationSection::Fonts) => parse_font_line,
            None => parse_nothing,
        };

        let Ok((_, block)) = parse_fn(line.as_str()) else {
            continue;
        };

        match block {
            AssCategory::Events(event) => match event.kind {
                SubStationEventKind::Dialogue => dialogue.push(event),
                SubStationEventKind::Picture => pictures.push(event),
                SubStationEventKind::Sound => sounds.push(event),
                SubStationEventKind::Movie => movies.push(event),
                SubStationEventKind::Command => commands.push(event),
            },
            AssCategory::Fonts(font) => raw_fonts.push(font.to_string()),
            AssCategory::Graphics(graphic) => raw_graphics.push(graphic.to_string()),
            AssCategory::ScriptInfo((key, value)) => {
                raw_script_info.insert(key.to_string(), value.to_string());
            }
            AssCategory::Styles(style) => styles.push(style),
        }
    }

    let fonts = match parse_fonts(raw_fonts.join("\n").as_str()) {
        Ok((_, fonts)) => fonts,
        Err(_) => Vec::new(),
    };

    let graphics = match parse_graphics(raw_graphics.join("\n").as_str()) {
        Ok((_, graphics)) => graphics,
        Err(_) => Vec::new(),
    };

    AssSubtitle::builder()
        .script_info(build_script_info(&raw_script_info))
        .dialogue(dialogue)
        .pictures(pictures)
        .sounds(sounds)
        .movies(movies)
        .commands(commands)
        .styles(styles)
        .fonts(fonts)
        .graphics(graphics)
        .build()
}
