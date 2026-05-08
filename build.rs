fn main() {
    slint_build::compile("ui/app.slint").unwrap();

    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("ccswap.ico");
        res.compile().unwrap();
    }
}
