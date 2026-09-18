fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../assets/haavk.ico");
        res.set("FileDescription", "HAAVK 曼德尔全域算力终端");
        res.set("ProductName", "HAAVK OS");
        res.set("ProductVersion", "0.1.0");
        res.set("FileVersion", "0.1.0");
        res.set("OriginalFilename", "haavk.exe");
        res.compile().expect("嵌入 HAAVK 图标失败");
    }
}
