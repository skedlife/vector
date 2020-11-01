use chrono::prelude::Local;
use fakedata_generator::{gen_http_method, gen_ipv4, gen_username};
use rand::{thread_rng, Rng};

pub fn apache_common_log_line() -> String {
    // Example log line:
    // 173.159.239.159 - schoen1464 [31/Oct/2020:19:06:10 -0700] "POST /wireless HTTP/2.0" 100 20815
    format!(
        "{} - {} [{}] \"{} {} {}\" {} {}",
        ipv4_address(),
        username(),
        now_apache_common(),
        http_method(),
        http_endpoint(),
        http_version(),
        http_code(),
        byte_size(),
    )
}

fn http_code() -> String {
    let codes: Vec<u16> = vec![
        200, 300, 301, 302, 304, 307, 400, 401, 403, 404, 410, 500, 501, 503, 550,
    ];

    random_from_vec::<u16>(codes).to_string()
}

fn byte_size() -> String {
    thread_rng().gen_range(50, 50000).to_string()
}

fn now_apache_common() -> String {
    Local::now().format("%d/%b/%Y:%T %z").to_string()
}

fn http_endpoint() -> String {
    let endpoints: Vec<&'static str> = vec!["/foo", "/bar"];
    random_from_vec::<&'static str>(endpoints).into()
}

fn http_method() -> String {
    gen_http_method()
}

fn http_version() -> String {
    let versions: Vec<&'static str> = vec!["HTTP/1.0", "HTTP/1.1", "HTTP/2.0"];
    random_from_vec::<&'static str>(versions).into()
}

fn ipv4_address() -> String {
    gen_ipv4()
}

fn username() -> String {
    gen_username()
}

// Helper functions
fn random_from_vec<T: Copy>(v: Vec<T>) -> T {
    v[thread_rng().gen_range(0, v.len())]
}
