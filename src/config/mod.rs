macro_rules! define_enum_and_to_string {
    ($enum_name:ident { $($variant_name:ident => $variant_str:expr),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $enum_name {
            $($variant_name),*
        }

        impl std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$variant_name => write!(f, "{}", $variant_str)),*
                }
            }
        }
    };
}

define_enum_and_to_string! {
    ImageKey {
        // 為什麼要演奏春日影
        WhyHaRuHiKaGe => "ui/特效圖片/為什麼要演奏春日影！.png",
        GenShinStart => "genshin-start.png"
    }
}
