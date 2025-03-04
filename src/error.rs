pub fn error(line: i32, message: &str) {
    report(line, "", message);
}

fn report(line: i32, where_: &str, message: &str) {
    eprintln!("[line {line}] Error{where_}: {message}");
    //hadError = true;
}
