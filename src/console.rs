use std::io::Write;

pub fn confirm(question: &str) -> bool {
    print!("{}", question);
    
    let mut answer_raw = String::new();
    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut answer_raw)
        .expect("Failed to read input");
    let answer = answer_raw.trim_end();
        
    answer == "y" || answer == "Y"
}