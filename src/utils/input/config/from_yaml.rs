use {
    crate::{
        replay::concrete::{
            ExchangeSession,
            GetNextObSnapshotDelay,
            TradedPairLifetime,
        },
        traded_pair::{TradedPair, TradedPairParser},
        types::{
            DateTime,
            Identifier,
            PriceStep,
        },
        utils::{
            ExpectWith,
            input::{
                config::{
                    from_structs::{OneTickReplayConfig, OneTickTradedPairReaderConfig},
                    from_yaml::{config_fields::*, yaml_utils::*},
                },
                one_tick::TrdPrlConfig,
            },
        },
    },
    csv::{ReaderBuilder, StringRecord},
    std::{
        collections::HashMap,
        fs::read_to_string,
        iter::once,
        path::Path,
        str::FromStr,
    },
    yaml_rust::{Yaml, yaml::Hash, YamlLoader},
};

#[cfg(test)]
mod tests;

mod yaml_utils
{
    use {
        crate::utils::ExpectWith,
        std::{path::Path, str::FromStr},
        yaml_rust::{Yaml, yaml::{Array, Hash}},
    };

    pub fn expect_yaml_hashmap<'a>(
        yml: &'a Yaml,
        path: &Path,
        get_current_section: impl Fn() -> String) -> &'a Hash
    {
        match yml {
            Yaml::Hash(map) => map,
            Yaml::BadValue => panic!(
                "{:?} does not have \"{}\" section", path, get_current_section()
            ),
            _ => panic!(
                "\"{}\" section of the {:?} YAML file should contain named entries. Got {:?}",
                get_current_section(), path, yml
            )
        }
    }

    pub fn try_expect_yaml_hashmap<'a>(
        yml: &'a Yaml,
        path: &Path,
        get_current_section: impl Fn() -> String) -> Option<&'a Hash>
    {
        match yml {
            Yaml::Hash(map) => Some(map),
            Yaml::BadValue => None,
            _ => panic!(
                "\"{}\" section of the {:?} YAML file should contain named entries. Got {:?}",
                get_current_section(), path, yml
            )
        }
    }

    pub fn expect_yaml_array<'a>(
        yml: &'a Yaml,
        path: &Path,
        get_current_section: impl Fn() -> String) -> &'a Array
    {
        match yml {
            Yaml::Array(arr) => arr,
            Yaml::BadValue => panic!(
                "{:?} does not have \"{}\" section", path, get_current_section()
            ),
            _ => panic!(
                "\"{}\" section of the {:?} YAML file should be an array of entries. Got {:?}",
                get_current_section(), path, yml
            )
        }
    }

    pub fn expect_yaml_string<'a>(
        yml: &'a Yaml,
        path: &Path,
        get_current_section: impl Fn() -> String) -> &'a String
    {
        match yml {
            Yaml::String(string) => string,
            Yaml::BadValue => panic!(
                "{:?} does not have \"{}\" section", path, get_current_section()
            ),
            _ => panic!(
                "\"{}\" section of the {:?} YAML file should be String. Got {:?}",
                get_current_section(), path, yml
            )
        }
    }

    pub fn expect_yaml_real<'a>(
        yml: &'a Yaml,
        path: &Path,
        get_current_section: impl Fn() -> String) -> &'a String
    {
        match yml {
            Yaml::Real(real) => real,
            Yaml::BadValue => panic!(
                "{:?} does not have \"{}\" section", path, get_current_section()
            ),
            _ => panic!(
                "\"{}\" section of the {:?} YAML file should be Real. Got {:?}",
                get_current_section(), path, yml
            )
        }
    }

    pub fn read_yaml_hashmap_field<'a>(
        map: &'a Hash,
        field: &str,
        path: &Path,
        get_current_section: impl Fn() -> String) -> &'a Yaml
    {
        try_read_yaml_hashmap_field(map, field).expect_with(
            || panic!(
                "\"{}\" section of the {:?} YAML file is not found",
                get_current_section(), path
            )
        )
    }

    pub fn try_read_yaml_hashmap_field<'a>(map: &'a Hash, field: &str) -> Option<&'a Yaml> {
        map.get(&Yaml::from_str(field))
    }

    #[derive(Debug, Clone, derive_more::From)]
    pub enum YamlValue {
        Real(f64),
        Integer(i64),
        String(String),
        Boolean(bool),
    }

    impl From<&str> for YamlValue {
        fn from(s: &str) -> Self { YamlValue::String(s.to_string()) }
    }

    impl From<&String> for YamlValue {
        fn from(s: &String) -> Self { YamlValue::String(s.to_string()) }
    }

    pub fn expect_yaml_value(yml: &Yaml, get_current_section: impl Fn() -> String) -> YamlValue
    {
        match yml {
            Yaml::Real(real) => f64::from_str(real)
                .expect_with(
                    || panic!("Section \"{}\". Cannot parse \"{}\" to f64",
                              get_current_section(),
                              real)
                )
                .into(),
            Yaml::Integer(integer) => (*integer).into(),
            Yaml::String(string) => string.into(),
            Yaml::Boolean(boolean) => (*boolean).into(),
            _ => panic!(
                "Section \"{}\" should contain values only. Got {:?}",
                get_current_section(), yml
            )
        }
    }
}

mod config_fields {
    /// Main sections
    pub const DEFAULTS: &str = "Defaults";
    pub const SIMULATION_TIME: &str = "Simulation Time";
    pub const EXCHANGES: &str = "Exchanges";
    pub const TRADED_PAIRS: &str = "Traded Pairs";

    /// Can be set as defaults
    pub const DATETIME_FORMAT: &str = "datetime_format";
    pub const CSV_SEP: &str = "csv_sep";
    pub const OPEN_COLNAME: &str = "open_colname";
    pub const CLOSE_COLNAME: &str = "close_colname";
    pub const DATETIME_COLNAME: &str = "datetime_colname";
    pub const REFERENCE_ORDER_ID_COLNAME: &str = "reference_order_id_colname";
    pub const ORDER_ID_COLNAME: &str = "order_id_colname";
    pub const SIZE_COLNAME: &str = "size_colname";
    pub const PRICE_COLNAME: &str = "price_colname";
    pub const BUY_SELL_FLAG_COLNAME: &str = "buy_sell_flag_colname";
    pub const START_COLNAME: &str = "start_colname";
    pub const STOP_COLNAME: &str = "stop_colname";

    /// Simulation time specific fields
    pub const START: &str = "start";
    pub const END: &str = "end";

    /// Exchanges specific fields
    pub const NAME: &str = "name";
    pub const SESSIONS: &str = "sessions";

    /// Exchange session specific fields
    pub const PATH: &str = "path";

    /// Traded Pairs specific fields
    pub const EXCHANGE: &str = "exchange";
    pub const KIND: &str = "kind";
    pub const QUOTED: &str = "quoted";
    pub const BASE: &str = "base";
    pub const PRICE_STEP: &str = "price_step";
    pub const ERR_LOG_FILE: &str = "err_log_file";
    pub const START_STOP_DATETIMES: &str = "start_stop_datetimes";
    pub const TRD: &str = "trd";
    pub const PRL: &str = "prl";

    /// TRD-PRL specific fields
    pub const PATH_LIST: &str = "path_list";
}

mod defaults {
    pub const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.f";
    pub const CSV_SEP: &str = ",";
}

pub fn parse_yaml<ExchangeID, Symbol, TPP, ObSnapshotDelay>(
    path: impl AsRef<Path>,
    _traded_pair_parser: TPP,
    ob_snapshot_delay_scheduler: ObSnapshotDelay,
) -> (
    Vec<ExchangeID>,
    OneTickReplayConfig<ExchangeID, Symbol, ObSnapshotDelay>,
    DateTime,
    DateTime
)
    where ExchangeID: Identifier + FromStr,
          Symbol: Identifier + FromStr,
          TPP: TradedPairParser<Symbol>,
          ObSnapshotDelay: GetNextObSnapshotDelay<ExchangeID, Symbol>
{
    const POSSIBLE_SECTIONS: [&str; 4] = [
        DEFAULTS,
        SIMULATION_TIME,
        EXCHANGES,
        TRADED_PAIRS
    ];

    let path = path.as_ref();
    let yml = read_to_string(path)
        .expect_with(|| panic!("Cannot read the following file: {:?}", path));
    let yml = YamlLoader::load_from_str(&yml)
        .expect_with(|| panic!("Bad YAML file: {:?}", path));
    let yml = &yml[0];

    let cwd = std::env::current_dir().expect("Cannot get current working directory");
    let parent_dir = path.parent().expect_with(
        || panic!("Cannot get parent directory of the {:?}", path)
    );
    std::env::set_current_dir(parent_dir).expect_with(
        || panic!("Cannot set current working directory to {:?}", parent_dir)
    );

    const GET_CURRENT_SECTION: fn() -> String = || "~".into();
    expect_yaml_hashmap(yml, path, GET_CURRENT_SECTION).keys().for_each(
        |key| {
            let key = expect_yaml_string(key, path, GET_CURRENT_SECTION);
            if !POSSIBLE_SECTIONS.contains(&key.as_str()) {
                panic!(
                    "\"{}\" cannot be present in the \"{}\" section. Possible keys: {:?}",
                    key, GET_CURRENT_SECTION(), POSSIBLE_SECTIONS
                )
            }
        }
    );

    let mut defaults = init_defaults();

    parse_defaults_section(yml, path, &mut defaults);
    let (start, end) = parse_simulation_time_section(yml, path, defaults.clone());

    let (exchanges, sessions): (_, Vec<_>) = parse_exchanges_section(yml, path, &defaults)
        .into_iter()
        .unzip();

    let (traded_pair_readers, start_stop_events): (Vec<_>, Vec<_>) = parse_traded_pairs_section::<
        ExchangeID, Symbol, TPP>(yml, path, defaults)
        .into_iter()
        .unzip();

    std::env::set_current_dir(&cwd).expect_with(
        || panic!("Cannot set current working directory to {:?}", cwd)
    );

    (
        exchanges,
        OneTickReplayConfig {
            start_dt: start,
            traded_pair_configs: traded_pair_readers,
            exchange_open_close_events: sessions.into_iter().flatten().collect(),
            traded_pair_creation_events: start_stop_events.into_iter().flatten().collect(),
            ob_snapshot_delay_scheduler,
        },
        start,
        end
    )
}

type Env = HashMap<String, YamlValue>;

fn init_defaults() -> Env {
    [DATETIME_FORMAT, CSV_SEP]
        .into_iter()
        .map(String::from)
        .zip([defaults::DATETIME_FORMAT.into(), defaults::CSV_SEP.into()])
        .collect()
}

fn update_env<const KEYS_NUM: usize>(
    map: &Hash,
    env: &mut Env,
    path: &Path,
    get_current_section: impl Fn() -> String,
    possible_keys: [&str; KEYS_NUM])
{
    map.into_iter().for_each(
        |(key, value)| {
            let key = expect_yaml_string(
                key, path, || format!("{} :: {:?}", get_current_section(), key),
            );
            if !possible_keys.contains(&key.as_str()) {
                panic!(
                    "\"{}\" cannot be present in the \"{}\" section. Possible keys: {:?}",
                    key, get_current_section(), possible_keys
                )
            }
            let value = expect_yaml_value(
                value, || format!("{} :: {}", get_current_section(), key),
            );
            env.insert(key.into(), value);
        }
    )
}

fn parse_defaults_section(yaml: &Yaml, path: &Path, defaults: &mut Env)
{
    const POSSIBLE_KEYS: [&str; 12] = [
        DATETIME_FORMAT,
        CSV_SEP,
        OPEN_COLNAME,
        CLOSE_COLNAME,
        DATETIME_COLNAME,
        REFERENCE_ORDER_ID_COLNAME,
        ORDER_ID_COLNAME,
        PRICE_COLNAME,
        SIZE_COLNAME,
        BUY_SELL_FLAG_COLNAME,
        START_COLNAME,
        STOP_COLNAME
    ];

    const SECTION: &str = DEFAULTS;
    const FULL_SECTION_PATH: fn() -> String = || SECTION.into();

    if let Some(map) = try_expect_yaml_hashmap(&yaml[SECTION], path, FULL_SECTION_PATH)
    {
        update_env(map, defaults, path, FULL_SECTION_PATH, POSSIBLE_KEYS)
    }
}

fn parse_simulation_time_section(
    yaml: &Yaml,
    path: &Path,
    mut env: Env) -> (DateTime, DateTime)
{
    const POSSIBLE_KEYS: [&str; 3] = [
        DATETIME_FORMAT,
        START,
        END,
    ];
    const SECTION: &str = SIMULATION_TIME;
    const FULL_SECTION_PATH: fn() -> String = || SECTION.into();

    update_env(
        expect_yaml_hashmap(&yaml[SECTION], path, FULL_SECTION_PATH),
        &mut env, path, FULL_SECTION_PATH, POSSIBLE_KEYS,
    );

    let field = DATETIME_FORMAT;
    let datetime_format = env
        .get(field)
        .expect_with(|| unreachable!("Section \"{}\" should contain \"{}\" value", SECTION, field));

    let get_current_section = || format!("{} :: {}", SECTION, field);
    let datetime_format = if let YamlValue::String(v) = datetime_format {
        v.as_str()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), datetime_format)
    };

    let field = START;
    let start = env.get(field).expect_with(
        || panic!("Section \"{}\" should contain \"{}\" value", SECTION, field)
    );

    let get_current_section = || format!("{} :: {}", SECTION, field);
    let start = if let YamlValue::String(start) = start {
        start.as_str()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), start)
    };
    let start = DateTime::parse_from_str(start, datetime_format).expect_with(
        || panic!(
            "Section \"{}\". Cannot parse to DateTime: \"{}\". Datetime format used: \"{}\"",
            get_current_section(),
            start,
            datetime_format
        )
    );

    let field = END;
    let end = env.get(field).expect_with(
        || panic!("Section \"{}\" should contain \"{}\" value", SECTION, field)
    );

    let get_current_section = || format!("{} :: {}", SECTION, field);
    let end = if let YamlValue::String(end) = end {
        end.as_str()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), end)
    };
    let end = DateTime::parse_from_str(end, datetime_format).expect_with(
        || panic!(
            "Section \"{}\". Cannot parse to DateTime: \"{}\". Datetime format used: \"{}\"",
            get_current_section(),
            start,
            datetime_format
        )
    );

    (start, end)
}

fn parse_exchanges_section<'a, ExchangeID: Identifier + FromStr>(
    yaml: &'a Yaml,
    path: &'a Path,
    env: &'a Env) -> impl 'a + IntoIterator<Item=(ExchangeID, Vec<ExchangeSession<ExchangeID>>)>
{
    const POSSIBLE_KEYS: [&str; 2] = [
        NAME,
        SESSIONS
    ];
    const SECTION: &str = EXCHANGES;
    const FULL_SECTION_PATH: fn() -> String = || SECTION.into();

    expect_yaml_array(&yaml[SECTION], path, FULL_SECTION_PATH).into_iter().zip(1..).map(
        |(exchange, i)| {
            let get_current_section = || format!("{} :: {}", SECTION, i);
            let exchange = expect_yaml_hashmap(exchange, path, get_current_section);

            for key in exchange.keys() {
                let get_current_section = || format!("{} :: {} :: {:?}", SECTION, i, key);
                let key = expect_yaml_string(key, path, get_current_section);
                if !POSSIBLE_KEYS.contains(&key.as_str()) {
                    panic!(
                        "\"{}\" cannot be present in the \"{}\" section. Possible keys: {:?}",
                        key, get_current_section(), POSSIBLE_KEYS
                    )
                }
            }

            let field = NAME;
            let full_section_path = || format!("{} :: {} :: {}", SECTION, i, field);
            let name = read_yaml_hashmap_field(exchange, field, path, full_section_path);
            let name = expect_yaml_string(name, path, full_section_path);
            let name = FromStr::from_str(name).expect_with(
                || panic!("Section \"{}\". Cannot parse \"{}\" to ExchangeID",
                          full_section_path(), name)
            );

            let field = SESSIONS;
            let full_section_path = || format!("{} :: {} :: {}", SECTION, i, field);
            let sessions = read_yaml_hashmap_field(exchange, field, path, full_section_path);
            let sessions = expect_yaml_hashmap(sessions, path, full_section_path);
            let sessions = parse_exchange_sessions(
                sessions, name, path, env.clone(), &full_section_path,
            );
            (name, sessions)
        }
    )
}

fn parse_exchange_sessions<ExchangeID: Identifier>(
    yaml: &Hash,
    name: ExchangeID,
    path: &Path,
    mut env: HashMap<String, YamlValue>,
    full_section_path: impl Copy + Fn() -> String) -> Vec<ExchangeSession<ExchangeID>>
{
    const POSSIBLE_KEYS: [&str; 5] = [
        PATH,
        OPEN_COLNAME,
        CLOSE_COLNAME,
        DATETIME_FORMAT,
        CSV_SEP
    ];

    update_env(yaml, &mut env, path, full_section_path, POSSIBLE_KEYS);

    let field = DATETIME_FORMAT;
    let datetime_format = env
        .get(field)
        .expect_with(
            || unreachable!(
                "Section \"{}\" should contain \"{}\" value", full_section_path(), field
            )
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let datetime_format = if let YamlValue::String(v) = datetime_format {
        v.as_str()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), datetime_format)
    };


    let field = CSV_SEP;
    let csv_sep = env
        .get(field)
        .expect_with(
            || unreachable!(
                "Section \"{}\" should contain \"{}\" value", full_section_path(), field
            )
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let csv_sep = if let YamlValue::String(v) = csv_sep {
        v.as_str()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), csv_sep)
    };
    if csv_sep.len() != 1 {
        panic!("\"{}\" should contain 1 character. Got {}", get_current_section(), csv_sep)
    }
    let csv_sep = *csv_sep.as_bytes().first().unwrap();


    let field = OPEN_COLNAME;
    let open_colname = env
        .get(field)
        .expect_with(
            || panic!(
                "Section \"{}\" should contain \"{}\" value", full_section_path(), field
            )
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let open_colname = if let YamlValue::String(v) = open_colname {
        v.as_str()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), open_colname)
    };


    let field = CLOSE_COLNAME;
    let close_colname = env
        .get(field)
        .expect_with(
            || panic!(
                "Section \"{}\" should contain \"{}\" value", full_section_path(), field
            )
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let close_colname = if let YamlValue::String(v) = close_colname {
        v.as_str()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), close_colname)
    };


    let field = PATH;
    let path = env
        .get(field)
        .expect_with(
            || panic!(
                "Section \"{}\" should contain \"{}\" value", full_section_path(), field
            )
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let path = if let YamlValue::String(v) = path {
        v.as_str()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), path)
    };


    let mut csv_reader = ReaderBuilder::new()
        .delimiter(csv_sep)
        .from_path(path)
        .expect_with(|| panic!("Cannot read the following file: {}", path));

    let header = csv_reader
        .headers()
        .expect_with(|| panic!("Cannot parse header of the CSV-file: {}", path));

    let mut open_colname_idx = None;
    let mut close_colname_idx = None;

    header.iter().enumerate().for_each(
        |(i, col)| {
            if col == open_colname {
                if open_colname_idx.is_none() {
                    open_colname_idx = Some(i)
                } else {
                    panic!("Duplicate column {} in the CSV-file {}", open_colname, path)
                }
            } else if col == close_colname {
                if close_colname_idx.is_none() {
                    close_colname_idx = Some(i)
                } else {
                    panic!("Duplicate column {} in the CSV-file {}", close_colname, path)
                }
            }
        }
    );
    let open_colname_idx = open_colname_idx.expect_with(
        || panic!("Cannot not find \"{}\" column in the CSV-file {}", open_colname, path)
    );
    let close_colname_idx = close_colname_idx.expect_with(
        || panic!("Cannot not find \"{}\" column in the CSV-file {}", close_colname, path)
    );

    let parse_record = |(record, i): (Result<StringRecord, _>, _)| {
        let record = record.expect_with(
            || panic!("Cannot parse {} line of the CSV-file {}", i, path)
        );
        let open_dt = record.get(open_colname_idx).expect_with(
            || panic!(
                "{} line of the CSV-file {} does not have value at the {} index",
                i, path, open_colname_idx
            )
        );
        let close_dt = record.get(close_colname_idx).expect_with(
            || panic!(
                "{} line of the CSV-file {} does not have value at the {} index",
                i, path, close_colname_idx
            )
        );
        if close_dt > open_dt {
            ExchangeSession {
                exchange_id: name,
                open_dt: DateTime::parse_from_str(open_dt, datetime_format).expect_with(
                    || panic!(
                        "{} line of the CSV-file {}. \
                        Cannot parse to DateTime: {}. Datetime format used: {}",
                        i, path,
                        open_dt,
                        datetime_format
                    )
                ),
                close_dt: DateTime::parse_from_str(close_dt, datetime_format).expect_with(
                    || panic!(
                        "{} line of the CSV-file {}. \
                        Cannot parse to DateTime: {}. Datetime format used: {}",
                        i, path,
                        close_dt,
                        datetime_format
                    )
                ),
            }
        } else {
            panic!(
                "{} line of the CSV-file {}. close_dt should be greater than open_dt",
                i, path
            )
        }
    };
    let mut record_iterator = csv_reader.records().zip(2..).map(parse_record);

    let first_record = record_iterator.next().expect_with(
        || panic!("CSV-file {} does not have any entries", path)
    );
    let mut last_dt = first_record.close_dt;

    once(first_record).chain(
        record_iterator.inspect(
            |session| if session.open_dt > last_dt {
                last_dt = session.close_dt
            } else {
                panic!(
                    "All entries in the CSV-file {} should be sorted \
                    in ascending order by time. \
                    I.e. each open_dt should be greater than the previous close_dt",
                    path
                )
            }
        )
    ).collect()
}

fn parse_traded_pairs_section<
    'a,
    ExchangeID: Identifier + FromStr,
    Symbol: Identifier + FromStr,
    TPParser: TradedPairParser<Symbol>
>(
    yaml: &'a Yaml,
    path: &'a Path,
    env: Env) -> impl 'a + IntoIterator<
    Item=(
        OneTickTradedPairReaderConfig<ExchangeID, Symbol>,
        Vec<TradedPairLifetime<ExchangeID, Symbol>>
    )
> {
    const POSSIBLE_KEYS: [&str; 9] = [
        EXCHANGE,
        KIND,
        QUOTED,
        BASE,
        PRICE_STEP,
        START_STOP_DATETIMES,
        ERR_LOG_FILE,
        TRD,
        PRL,
    ];
    const SECTION: &str = "Traded Pairs";
    const FULL_SECTION_PATH: fn() -> String = || SECTION.into();

    expect_yaml_array(&yaml[SECTION], path, FULL_SECTION_PATH).into_iter().zip(1..).map(
        move |(map, i)| {
            let get_current_section = || format!("{} :: {}", SECTION, i);
            let map = expect_yaml_hashmap(map, path, get_current_section);
            for key in map.keys() {
                let get_current_section = || format!("{} :: {} :: {:?}", SECTION, i, key);
                let key = expect_yaml_string(key, path, get_current_section);
                if !POSSIBLE_KEYS.contains(&key.as_str()) {
                    panic!(
                        "\"{}\" cannot be present in the \"{}\" section. Possible keys: {:?}",
                        key, get_current_section(), POSSIBLE_KEYS
                    )
                }
            }

            let field = EXCHANGE;
            let full_section_path = || format!("{} :: {} :: {}", SECTION, i, field);
            let exchange = read_yaml_hashmap_field(map, field, path, full_section_path);
            let exchange = expect_yaml_string(exchange, path, full_section_path);
            let exchange = FromStr::from_str(exchange).expect_with(
                || panic!("Section \"{}\". Cannot parse \"{}\" to ExchangeID",
                          full_section_path(), exchange)
            );

            let field = KIND;
            let full_section_path = || format!("{} :: {} :: {}", SECTION, i, field);
            let kind = read_yaml_hashmap_field(map, field, path, full_section_path);
            let kind = expect_yaml_string(kind, path, full_section_path);

            let field = QUOTED;
            let full_section_path = || format!("{} :: {} :: {}", SECTION, i, field);
            let quoted = read_yaml_hashmap_field(map, field, path, full_section_path);
            let quoted = expect_yaml_string(quoted, path, full_section_path);

            let field = BASE;
            let full_section_path = || format!("{} :: {} :: {}", SECTION, i, field);
            let base = read_yaml_hashmap_field(map, field, path, full_section_path);
            let base = expect_yaml_string(base, path, full_section_path);

            let field = PRICE_STEP;
            let full_section_path = || format!("{} :: {} :: {}", SECTION, i, field);
            let price_step = read_yaml_hashmap_field(map, field, path, full_section_path);
            let price_step = expect_yaml_real(price_step, path, full_section_path);
            let price_step: PriceStep = f64::from_str(price_step).expect_with(
                || panic!("Section \"{}\". Cannot parse to f64: {}",
                          full_section_path(), price_step)
            ).into();

            let field = ERR_LOG_FILE;
            let full_section_path = || format!("{} :: {} :: {}", SECTION, i, field);
            let err_log_file = try_read_yaml_hashmap_field(map, field);
            let err_log_file = if let Some(err_log_file) = err_log_file {
                let err_log_file = expect_yaml_string(err_log_file, path, full_section_path);
                let err_log_file = Path::new(err_log_file);
                if err_log_file.is_relative() {
                    let result = path.parent()
                        .expect_with(
                            || unreachable!("Cannot get parent directory of the {:?}", path)
                        )
                        .join(err_log_file);
                    Some(Box::from(result))
                } else {
                    Some(Box::from(err_log_file))
                }
            } else {
                None
            };

            let traded_pair = TPParser::parse(kind, quoted, base);

            let field = START_STOP_DATETIMES;
            let full_section_path = || format!("{} :: {} :: {}", SECTION, i, field);
            let trade_start_stops = read_yaml_hashmap_field(map, field, path, full_section_path);
            let trade_start_stops = expect_yaml_hashmap(trade_start_stops, path, full_section_path);
            let trade_start_stops = parse_trade_start_stops(
                trade_start_stops, traded_pair, price_step, exchange,
                env.clone(), path, full_section_path,
            );

            let traded_pair_reader = gen_traded_pair_reader(
                map, traded_pair, price_step, exchange,
                env.clone(), path, get_current_section, err_log_file,
            );

            (traded_pair_reader, trade_start_stops)
        }
    )
}

fn parse_trade_start_stops<
    ExchangeID: Identifier,
    Symbol: Identifier,
>(
    map: &Hash,
    traded_pair: TradedPair<Symbol>,
    price_step: PriceStep,
    exchange_id: ExchangeID,
    mut env: HashMap<String, YamlValue>,
    path: &Path,
    get_current_section: impl Fn() -> String) -> Vec<TradedPairLifetime<ExchangeID, Symbol>>
{
    const POSSIBLE_KEYS: [&str; 5] = [
        PATH,
        START_COLNAME,
        STOP_COLNAME,
        DATETIME_FORMAT,
        CSV_SEP
    ];
    const SECTION: &str = START_STOP_DATETIMES;
    let full_section_path = || format!("{} :: {}", get_current_section(), SECTION);

    update_env(map, &mut env, path, full_section_path, POSSIBLE_KEYS);


    let field = DATETIME_FORMAT;
    let datetime_format = env
        .get(field)
        .expect_with(
            || unreachable!(
                "Section \"{}\" should contain \"{}\" value", full_section_path(), field
            )
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let datetime_format = if let YamlValue::String(v) = datetime_format {
        v.as_str()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), datetime_format)
    };


    let field = CSV_SEP;
    let csv_sep = env
        .get(field)
        .expect_with(
            || unreachable!(
                "Section \"{}\" should contain \"{}\" value", full_section_path(), field
            )
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let csv_sep = if let YamlValue::String(v) = csv_sep {
        v.as_str()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), csv_sep)
    };
    if csv_sep.len() != 1 {
        panic!("\"{}\" should contain 1 character. Got {}", get_current_section(), csv_sep)
    }
    let csv_sep = *csv_sep.as_bytes().first().unwrap();


    let field = START_COLNAME;
    let start_colname = env
        .get(field)
        .expect_with(
            || panic!(
                "Section \"{}\" should contain \"{}\" value", full_section_path(), field
            )
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let start_colname = if let YamlValue::String(v) = start_colname {
        v.as_str()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), start_colname)
    };


    let field = STOP_COLNAME;
    let stop_colname = env
        .get(field)
        .expect_with(
            || panic!("Section \"{}\" should contain \"{}\" value", full_section_path(), field)
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let stop_colname = if let YamlValue::String(v) = stop_colname {
        v.as_str()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), stop_colname)
    };


    let field = PATH;
    let path = env
        .get(field)
        .expect_with(
            || panic!("Section \"{}\" should contain \"{}\" value", full_section_path(), field)
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let path = if let YamlValue::String(v) = path {
        v.as_str()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), path)
    };


    let mut csv_reader = ReaderBuilder::new()
        .delimiter(csv_sep)
        .from_path(path)
        .expect_with(|| panic!("Cannot read the following file: {}", path));

    let header = csv_reader
        .headers()
        .expect_with(|| panic!("Cannot parse header of the CSV-file: {}", path));

    let mut start_colname_idx = None;
    let mut stop_colname_idx = None;

    header.iter().enumerate().for_each(
        |(i, col)| {
            if col == start_colname {
                if start_colname_idx.is_none() {
                    start_colname_idx = Some(i)
                } else {
                    panic!("Duplicate column {} in the CSV-file {}", start_colname, path)
                }
            } else if col == stop_colname {
                if stop_colname_idx.is_none() {
                    stop_colname_idx = Some(i)
                } else {
                    panic!("Duplicate column {} in the CSV-file {}", stop_colname, path)
                }
            }
        }
    );
    let start_colname_idx = start_colname_idx.expect_with(
        || panic!("Cannot not find {} in the CSV-file {}", start_colname, path)
    );
    let stop_colname_idx = stop_colname_idx.expect_with(
        || panic!("Cannot not find {} in the CSV-file {}", stop_colname, path)
    );

    let mut already_non_stoppable = false;
    let parse_record = |(record, i): (Result<StringRecord, _>, _)| {
        if already_non_stoppable {
            panic!(
                "{} line of the CSV-file {}. Cannot have entries after entry without stop_dt",
                i, path
            )
        }
        let record = record.expect_with(
            || panic!("Cannot parse {} line of the CSV-file {}", i, path)
        );
        let start_dt = record.get(start_colname_idx).expect_with(
            || panic!(
                "{} line of the CSV-file {} does not have value at the {} index",
                i, path, start_colname_idx
            )
        );
        let start_dt = DateTime::parse_from_str(start_dt, datetime_format).expect_with(
            || panic!(
                "{} line of the CSV-file {}. \
                Cannot parse to DateTime: {}. Datetime format used: {}",
                i, path,
                start_dt,
                datetime_format
            )
        );
        let stop_dt = record.get(stop_colname_idx).expect_with(
            || panic!(
                "{} line of the CSV-file {} does not have value at the {} index",
                i, path, stop_colname_idx
            )
        );
        let stop_dt = if !stop_dt.is_empty() {
            let stop_dt = DateTime::parse_from_str(stop_dt, datetime_format).expect_with(
                || panic!(
                    "{} line of the CSV-file {}. \
                    Cannot parse to DateTime: {}. Datetime format used: {}",
                    i, path,
                    stop_dt,
                    datetime_format
                )
            );
            if stop_dt > start_dt {
                Some(stop_dt)
            } else {
                panic!(
                    "{} line of the CSV-file {}. stop_dt should be greater than start_dt",
                    i, path
                )
            }
        } else {
            already_non_stoppable = true;
            None
        };
        TradedPairLifetime {
            exchange_id,
            traded_pair,
            price_step,
            start_dt,
            stop_dt,
        }
    };
    let mut records_iterator = csv_reader.records().zip(2..).map(parse_record);
    let first_lifetime = records_iterator
        .next()
        .expect_with(|| panic!("CSV-file {} does not have any entries", path));
    let mut last_dt = if let Some(stop_dt) = first_lifetime.stop_dt {
        stop_dt
    } else {
        first_lifetime.start_dt
    };
    once(first_lifetime).chain(
        records_iterator.inspect(
            |lifetime| if lifetime.start_dt > last_dt {
                if let Some(stop_dt) = lifetime.stop_dt {
                    last_dt = stop_dt
                }
            } else {
                panic!(
                    "All entries in the CSV-file {} should be sorted \
                    in ascending order by time. \
                    I.e. each start_dt should be greater than the previous stop_dt",
                    path
                )
            }
        )
    ).collect()
}

fn gen_traded_pair_reader<
    ExchangeID: Identifier,
    Symbol: Identifier
>(
    map: &Hash,
    traded_pair: TradedPair<Symbol>,
    price_step: PriceStep,
    exchange_id: ExchangeID,
    env: HashMap<String, YamlValue>,
    path: &Path,
    get_current_section: impl Fn() -> String,
    err_log_file: Option<Box<Path>>) -> OneTickTradedPairReaderConfig<ExchangeID, Symbol>
{
    let field = TRD;
    let full_section_path = || format!("{} :: {}", get_current_section(), field);
    let trd = read_yaml_hashmap_field(map, field, path, full_section_path);
    let trd = expect_yaml_hashmap(trd, path, full_section_path);

    let (trd_files, trd_parsing_info) = gen_trd_prl_config::<_, true>(
        trd, env.clone(), price_step, path, full_section_path,
    );

    let field = PRL;
    let full_section_path = || format!("{} :: {}", get_current_section(), field);
    let prl = read_yaml_hashmap_field(map, field, path, full_section_path);
    let prl = expect_yaml_hashmap(prl, path, full_section_path);

    let (prl_files, prl_parsing_info) = gen_trd_prl_config::<_, false>(
        prl, env, price_step, path, full_section_path,
    );

    OneTickTradedPairReaderConfig {
        exchange_id,
        traded_pair,
        prl_files,
        prl_args: prl_parsing_info,
        trd_files,
        trd_args: trd_parsing_info,
        err_log_file,
    }
}

const fn get_order_id_colname<const IS_TRD: bool>() -> &'static str {
    if IS_TRD {
        REFERENCE_ORDER_ID_COLNAME
    } else {
        ORDER_ID_COLNAME
    }
}

fn gen_trd_prl_config<F: Fn() -> String, const IS_TRD: bool>(
    map: &Hash,
    mut env: HashMap<String, YamlValue>,
    price_step: PriceStep,
    path: &Path,
    full_section_path: F) -> (Box<Path>, TrdPrlConfig)
{
    let order_id_colname = get_order_id_colname::<IS_TRD>();
    let possible_keys = [
        PATH_LIST,
        DATETIME_FORMAT,
        CSV_SEP,
        OPEN_COLNAME,
        CLOSE_COLNAME,
        DATETIME_COLNAME,
        order_id_colname,
        PRICE_COLNAME,
        SIZE_COLNAME,
        BUY_SELL_FLAG_COLNAME
    ];

    update_env(map, &mut env, path, &full_section_path, possible_keys);


    let field = DATETIME_FORMAT;
    let datetime_format = env
        .get(field)
        .expect_with(
            || unreachable!(
                "Section \"{}\" should contain \"{}\" value", full_section_path(), field
            )
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let datetime_format = if let YamlValue::String(v) = datetime_format {
        v.to_string()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), datetime_format)
    };


    let field = CSV_SEP;
    let csv_sep = env
        .get(field)
        .expect_with(
            || unreachable!(
                "Section \"{}\" should contain \"{}\" value", full_section_path(), field
            )
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let csv_sep = if let YamlValue::String(v) = csv_sep {
        v.as_str()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), csv_sep)
    };
    if csv_sep.len() != 1 {
        panic!("\"{}\" should contain 1 character. Got {}", get_current_section(), csv_sep)
    }
    let csv_sep = *csv_sep.as_bytes().first().unwrap() as char;


    let field = DATETIME_COLNAME;
    let datetime_colname = env
        .get(field)
        .expect_with(
            || panic!("Section \"{}\" should contain \"{}\" value", full_section_path(), field)
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let datetime_colname = if let YamlValue::String(v) = datetime_colname {
        v.to_string()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), datetime_colname)
    };


    let field = order_id_colname;
    let order_id_colname = env.get(field)
        .expect_with(
            || panic!("Section \"{}\" should contain \"{}\" value", get_current_section(), field)
        );
    let order_id_colname = if let YamlValue::String(v) = order_id_colname {
        v.to_string()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", field, order_id_colname)
    };


    let field = PRICE_COLNAME;
    let price_colname = env
        .get(field)
        .expect_with(
            || panic!("Section \"{}\" should contain \"{}\" value", full_section_path(), field)
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let price_colname = if let YamlValue::String(v) = price_colname {
        v.to_string()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), price_colname)
    };


    let field = SIZE_COLNAME;
    let size_colname = env
        .get(field)
        .expect_with(
            || panic!("Section \"{}\" should contain \"{}\" value", full_section_path(), field)
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let size_colname = if let YamlValue::String(v) = size_colname {
        v.to_string()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), size_colname)
    };


    let field = BUY_SELL_FLAG_COLNAME;
    let buy_sell_flag_colname = env
        .get(field)
        .expect_with(
            || panic!("Section \"{}\" should contain \"{}\" value", full_section_path(), field)
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let buy_sell_flag_colname = if let YamlValue::String(v) = buy_sell_flag_colname {
        v.to_string()
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), buy_sell_flag_colname)
    };


    let field = PATH_LIST;
    let path_list = env
        .get(field)
        .expect_with(
            || panic!("Section \"{}\" should contain \"{}\" value", full_section_path(), field)
        );

    let get_current_section = || format!("{} :: {}", full_section_path(), field);
    let path_list = if let YamlValue::String(v) = path_list {
        Path::new(v)
    } else {
        panic!("\"{}\" should be String. Got: {:?}", get_current_section(), path_list)
    };
    let path_list = if path_list.is_relative() {
        let result = path.parent()
            .expect_with(|| unreachable!("Cannot get parent directory of the {:?}", path))
            .join(path_list);
        Box::from(result)
    } else {
        Box::from(path_list)
    };


    let info = TrdPrlConfig {
        datetime_colname,
        order_id_colname,
        price_colname,
        size_colname,
        buy_sell_flag_colname,
        datetime_format,
        csv_sep,
        price_step: price_step.into(),
    };

    (path_list, info)
}