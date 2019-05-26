use smithay_client_toolkit::window::{ButtonState, Theme};

pub struct WaylandTheme {
    /// Primary color when the window is focused
    pub primary_active: [u8; 4],
    /// Primary color when the window is unfocused
    pub primary_inactive: [u8; 4],
    /// Secondary color when the window is focused
    pub secondary_active: [u8; 4],
    /// Secondary color when the window is unfocused
    pub secondary_inactive: [u8; 4],
    /// Close button color when hovered over
    pub close_button_hovered: [u8; 4],
    /// Close button color
    pub close_button: [u8; 4],
    /// Close button fg color when hovered over
    pub close_button_icon_hovered: [u8; 4],
    /// Close button fg color
    pub close_button_icon: [u8; 4],
    /// Close button color when hovered over
    pub maximize_button_hovered: [u8; 4],
    /// Maximize button color
    pub maximize_button: [u8; 4],
    /// Minimize button color when hovered over
    pub minimize_button_hovered: [u8; 4],
    /// Minimize button color
    pub minimize_button: [u8; 4],
}

impl Theme for WaylandTheme {
    fn get_primary_color(&self, active: bool) -> [u8; 4] {
        if active {
            self.primary_active
        } else {
            self.primary_inactive
        }
    }

    // Used for division line
    fn get_secondary_color(&self, active: bool) -> [u8; 4] {
        if active {
            self.secondary_active
        } else {
            self.secondary_inactive
        }
    }

    fn get_close_button_color(&self, state: ButtonState) -> [u8; 4] {
        match state {
            ButtonState::Hovered => self.close_button_hovered,
            ButtonState::Idle => self.close_button,
            _ => self.close_button,
        }
    }

    fn get_close_button_icon_color(&self, state: ButtonState) -> [u8; 4] {
        match state {
            ButtonState::Hovered => self.close_button_icon_hovered,
            ButtonState::Idle => self.close_button_icon,
            _ => self.close_button,
        }
    }

    fn get_maximize_button_color(&self, state: ButtonState) -> [u8; 4] {
        match state {
            ButtonState::Hovered => self.maximize_button_hovered,
            _ => self.maximize_button,
        }
    }

    fn get_minimize_button_color(&self, state: ButtonState) -> [u8; 4] {
        match state {
            ButtonState::Hovered => self.minimize_button_hovered,
            _ => self.minimize_button,
        }
    }
}
