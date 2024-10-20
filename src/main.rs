use std::io;
use std::io::{Read, Write, BufReader};
use std::net::{TcpListener, TcpStream};

/// Represents an end user or client connecting to the server
struct Client;

/// Represents a computer hosting the web application
struct Server {
    /// The TCP listener for incoming connections
    connection: TcpListener,
}

impl Server {
    /// Creates a new Server instance
    ///
    /// # Arguments
    ///
    /// * `address` - A string slice that holds the IP address and port to bind to
    ///
    /// # Returns
    ///
    /// A new Server instance
        fn new(address: &str) -> io::Result<Server> {
        // Bind to the specified address and unwrap the result
        let listener = TcpListener::bind(address)?;
        Ok(Server {
            connection: listener
        })
    }
}

/// Represents HTTP methods
#[derive(Debug)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

// Type alias for headers, using a HashMap to store multiple values per header
type Headers = std::collections::HashMap<String, Vec<String>>;

/// Represents an HTTP request sent from the Client
#[derive(Debug)]
struct Request {
    // TODO: Add version field (e.g., HTTP/1.0, HTTP/1.1, HTTP/2.0)

    /// The requested resource path
    resource: String,

    /// The HTTP method used in the request
    method: HttpMethod,

    /// Headers included in the request
    headers: Headers,

    /// The body of the request
    body: Vec<u8>,
}

/// Reads a single header line from the given BufReader
///
/// # Arguments
///
/// * `stream` - A mutable reference to a BufReader<TcpStream>
///
/// # Returns
///
/// An io::Result containing the header line as a String
fn read_header_line(stream: &mut BufReader<TcpStream>) -> io::Result<String> {
    let mut buf: Vec<u8> = Vec::with_capacity(0x1000);

    // Read bytes until we encounter a newline character
    while let Some(Ok(byte)) = stream.bytes().next() {
        if byte == b'\n' {
            // Remove trailing carriage return if present
            if buf.ends_with(b"\r") {
                buf.pop();
            }

            // Convert the buffer to a UTF-8 string
            let header_line = String::from_utf8(buf)
                    .map_err(|_| { io::Error::new(io::ErrorKind::InvalidData, "Not an HTTP header") })?;
            return Ok(header_line);
        }

        buf.push(byte);
    }

    // If we reach this point, the client aborted the connection early
    Err(io::Error::new(io::ErrorKind::ConnectionAborted, "client aborted early"))
}


impl Request {
    /// Creates a new Request instance from a TcpStream
    ///
    /// # Arguments
    ///
    /// * `stream` - A BufReader<TcpStream> containing the raw HTTP request
    ///
    /// # Returns
    ///
    /// An io::Result containing the parsed Request
    fn new(mut stream: BufReader<TcpStream>) -> io::Result<Request> {
        // Read the first line of the HTTP request (e.g., "GET /index.html HTTP/1.1")
        let http_metadata = read_header_line(&mut stream)?;

        eprintln!("{http_metadata}");

        let mut parts = http_metadata.split_ascii_whitespace();

        // Parse the HTTP method
        let method = match parts.next().unwrap() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "DELETE" => HttpMethod::Delete,
            "PUT" => HttpMethod::Put,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "unsupported HTTP method"))
        };

        // Parse the requested resource
        let resource = parts.next().unwrap().to_string();

        // TODO: Parse and store the HTTP version
        let _version = parts.next();

        let mut headers = Headers::new();

        // Parse headers
        loop {
            let line = read_header_line(&mut stream)?;
            if line.is_empty() {
                break;
            }

            let mut parts = line.split(": ");
            let name = parts.next().unwrap().to_string();
            let value = parts.next().unwrap().to_string();

            // Add the header to our Headers HashMap
            let slot_for_value = headers
                .entry(name)
                .or_insert_with(|| { Vec::with_capacity(1) });
            slot_for_value.push(value);
        }

        // Read the request body
        let mut body = Vec::with_capacity(0x10000);
        let _ = stream.read(&mut body)?;

        Ok(Request {
            resource,
            method,
            headers,
            body,
        })
    }
}

/// Represents an HTTP response sent from the Server
struct Response;

fn main() -> io::Result<()> {
    let port = 8081; // Use a different port
    let server = Server::new(&format!("0.0.0.0:{}", port))?;
    println!("Server listening on http://localhost:{}", port);

    for stream in server.connection.incoming().flatten() {
        if let Err(e) = handle_connection(stream) {
            eprintln!("Error handling connection: {}", e);
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    let reader = BufReader::new(stream.try_clone()?);
    let request = Request::new(reader)?;
    
    println!("{:?}", request);

    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nHello, World!";
    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}
