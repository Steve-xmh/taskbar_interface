fn main() {
    windows::build!(
        Windows::Win32::System::Com::*,
        Windows::Win32::System::SystemServices::*,
        Windows::Win32::System::Diagnostics::Debug::*,
        Windows::Win32::UI::Shell::*,
    );
}
