use curl::easy::Easy;
use std::sync::{Mutex, Arc};
use std::thread;
use std::io::prelude::*;
use std::env;
use std::collections::{HashMap,VecDeque};

fn load_url(url: String) -> String {
    let mut req = Easy::new();
    
    match req.url(url.as_str()) {
        Ok(()) => {},
        Err(e) => {
            println!("Failed with error {}",e); 
        }
    }
    
    let mut buffer = Vec::new();
    
    {
        let mut req2 = req.transfer();
        req2.write_function(|x| {
            // println!("Recieved {} bytes",x.len());
            buffer.extend_from_slice(x);
            Ok(x.len())
        }).unwrap();
        
        req2.perform();
    } 

    let buffer2 = buffer.clone();
    String::from_utf8_lossy(buffer2.as_slice()).to_string()
}

fn parse_links(data: String) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    let mut chars = data.chars();

    let mut prev: Option<char> = None;
    

    while let Some(curr) = chars.next() {
        if curr == 'a' && prev == Some('<') {
            let mut passed: String = prev.unwrap().to_string();
            passed.push(curr);

            while let Some(temp) = chars.next() {
                passed.push(temp);
                if temp == '>' {
                
                    let tag: Vec<char> = passed.chars().collect();

                    if tag.get(3) == Some(&'h') 
                        && tag.get(10) == Some(&'w')
                        && tag.get(11) == Some(&'i') {

                            let s: Vec<char> = 
                                passed
                                .chars()
                                .skip(15)
                                .take_while(|c| c != &'"')
                                .collect();

                            let link = 
                                "https://en.wikipedia.org/wiki/".to_string() + 
                                s.iter().fold(String::new(), |a,b| {
                                    a+b.to_string().as_str()
                                }).as_str();
                            res.push(link);
                    }
                    break;
                }
            }
        }
        prev = Some(curr);
    }
    res
}

fn gogo(start: String, goal: String) {
    let mut temp: VecDeque<String> = VecDeque::new();
    temp.push_back(start.clone());

    let q: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(
        temp.clone()
    ));

    let active = Arc::new(Mutex::new(0));

    let backtrack = Arc::new(Mutex::new(HashMap::new()));

    let processed = Arc::new(Mutex::new(0));

    let found = Arc::new(Mutex::new(false));
    
    let mut threads = vec![];
    
    for _ in 0..2000 { // make a few workers
        let curr = Arc::clone(&active);
        let curr_q = Arc::clone(&q);
        let bc = Arc::clone(&backtrack);
        let p = Arc::clone(&processed);
        let f = Arc::clone(&found);
        
        let go = goal.clone();
        let st = start.clone();

        threads.push(thread::spawn(move || {
            loop {
                {
                    if *f.lock().unwrap() {
                        println!("{}/2000 threads active",curr.lock().unwrap());
                        break;
                    }
                }

                let mut key: Option<String> = None;
                {
                    let mut r = curr_q.lock().unwrap();
                    let c = curr.lock().unwrap();
                    if r.len() > 0 { // if there is an avaliable one
                        // do stuff with x
                        key = r.pop_front();
                    } else if *c == 0 { // if none of the others have values
                        break;
                    } else {
                        continue;
                    }
                }

                if let Some(k) = key {
                    {
                        let mut pr = p.lock().unwrap();
                        *pr+=1;
                        if *pr%1000==0 { 
                            println!("Processed {} with {}", pr,k);
                        }
                    }
                    {
                        let mut r = curr.lock().unwrap();
                        *r += 1;
                    }
                    
                    let text = load_url(k.clone());
                    let links = parse_links(text);

                    { // add new to q
                        let mut r = curr_q.lock().unwrap();
                        let mut dict = bc.lock().unwrap();

                        for x in links {
                            if !dict.contains_key(&x) {
                                r.push_back(x.clone());
                                dict.insert(x.clone(), k.clone());
                                if x == go {
                                    *f.lock().unwrap() = true;
                                    r.clear();
                                    println!("Found!");
                                    break;    
                                }

                            }
                        }
                    }
                    {
                        let mut r = curr.lock().unwrap();
                        *r -= 1;
                    }
                }


            }
        }));
    }
    for i in threads {
        match i.join() {
            Ok(_) => {}
            Err(_) =>{
                println!("died");
            
            }
        };
    }
    println!("Exited");
    let mut path: Vec<String> = Vec::new();

    let mut curr = goal;
    let dict = backtrack.lock().unwrap();
    loop {
        path.push(curr.to_string());
        if curr == start {
            break;
        }
        let prev = dict.get(&curr.to_string()).unwrap();
        curr=prev.clone();
    }
    println!("------ PATH -----");    
    for i in path.iter().rev() {
        println!("{}",i);
    }
    
}
fn main() {
    // let html = load_url("https://en.wikipedia.org/wiki/Barack_Obama".to_string());

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("Run ./wikipedia <source link> <goal link>");
    } else {
        let source = args.get(1).unwrap().clone();
        let dest = args.get(2).unwrap().clone();
        gogo(source,dest);
    }
    // parse_links(html);
}
