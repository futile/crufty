pub mod aabb {
    use crate::na::Point2;
    use crate::nc::bounding_volume::AABB;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct Bounds {
        mins: Point2<f32>,
        maxs: Point2<f32>,
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<AABB<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Bounds { mins, maxs } = Bounds::deserialize(deserializer)?;

        Ok(AABB::new(mins, maxs))
    }

    pub fn serialize<S>(aabb: &AABB<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bounds = Bounds {
            mins: *aabb.mins(),
            maxs: *aabb.maxs(),
        };

        bounds.serialize(serializer)
    }
}

pub mod cuboid {
    use crate::na::Vector2;
    use crate::nc::shape::Cuboid;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Cuboid<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vector2::deserialize(deserializer).map(Cuboid::new)
    }

    pub fn serialize<S>(cuboid: &Cuboid<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        cuboid.half_extents().serialize(serializer)
    }
}

pub mod virtual_key_code {
    use glium::glutin::VirtualKeyCode;

    // TODO: use winit's/glutin's native "serde" feature
    // We can't do that because glium doesn't re-export the feature
    #[derive(Serialize, Deserialize)]
    #[serde(remote = "VirtualKeyCode")]
    pub enum VirtualKeyCodeDef {
        /// The '1' key over the letters.
        Key1,
        /// The '2' key over the letters.
        Key2,
        /// The '3' key over the letters.
        Key3,
        /// The '4' key over the letters.
        Key4,
        /// The '5' key over the letters.
        Key5,
        /// The '6' key over the letters.
        Key6,
        /// The '7' key over the letters.
        Key7,
        /// The '8' key over the letters.
        Key8,
        /// The '9' key over the letters.
        Key9,
        /// The '0' key over the 'O' and 'P' keys.
        Key0,

        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I,
        J,
        K,
        L,
        M,
        N,
        O,
        P,
        Q,
        R,
        S,
        T,
        U,
        V,
        W,
        X,
        Y,
        Z,

        /// The Escape key, next to F1.
        Escape,

        F1,
        F2,
        F3,
        F4,
        F5,
        F6,
        F7,
        F8,
        F9,
        F10,
        F11,
        F12,
        F13,
        F14,
        F15,
        F16,
        F17,
        F18,
        F19,
        F20,
        F21,
        F22,
        F23,
        F24,

        /// Print Screen/SysRq.
        Snapshot,
        /// Scroll Lock.
        Scroll,
        /// Pause/Break key, next to Scroll lock.
        Pause,

        /// `Insert`, next to Backspace.
        Insert,
        Home,
        Delete,
        End,
        PageDown,
        PageUp,

        Left,
        Up,
        Right,
        Down,

        /// The Backspace key, right over Enter.
        // TODO: rename
        Back,
        /// The Enter key.
        Return,
        /// The space bar.
        Space,

        /// The "Compose" key on Linux.
        Compose,

        Caret,

        Numlock,
        Numpad0,
        Numpad1,
        Numpad2,
        Numpad3,
        Numpad4,
        Numpad5,
        Numpad6,
        Numpad7,
        Numpad8,
        Numpad9,

        AbntC1,
        AbntC2,
        Add,
        Apostrophe,
        Apps,
        At,
        Ax,
        Backslash,
        Calculator,
        Capital,
        Colon,
        Comma,
        Convert,
        Decimal,
        Divide,
        Equals,
        Grave,
        Kana,
        Kanji,
        LAlt,
        LBracket,
        LControl,
        LShift,
        LWin,
        Mail,
        MediaSelect,
        MediaStop,
        Minus,
        Multiply,
        Mute,
        MyComputer,
        NavigateForward,  // also called "Prior"
        NavigateBackward, // also called "Next"
        NextTrack,
        NoConvert,
        NumpadComma,
        NumpadEnter,
        NumpadEquals,
        OEM102,
        Period,
        PlayPause,
        Power,
        PrevTrack,
        RAlt,
        RBracket,
        RControl,
        RShift,
        RWin,
        Semicolon,
        Slash,
        Sleep,
        Stop,
        Subtract,
        Sysrq,
        Tab,
        Underline,
        Unlabeled,
        VolumeDown,
        VolumeUp,
        Wake,
        WebBack,
        WebFavorites,
        WebForward,
        WebHome,
        WebRefresh,
        WebSearch,
        WebStop,
        Yen,
        Copy,
        Paste,
        Cut,
    }
}
