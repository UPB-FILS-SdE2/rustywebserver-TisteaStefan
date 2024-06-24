[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-22041afd0340ce965d47ae6ef1cefeee28c7c493a6346c4f15d667ab976d596c.svg)](https://classroom.github.com/a/TXciPqtn)
# Rustwebserver

Detail the homework implementation.


Main Function:

Argument Parsing: Retrieves the port number and root folder from the command line.
Startup Information: Prints the root folder and port to the console.
TCP Listener: Binds to the specified port and listens for incoming connections.
Connection Handling: Spawns a new thread for each incoming connection, calling handle_connection.
handle_connection Function:

Request Parsing: Reads the HTTP request, extracts the method, path, and headers.
File Path Construction: Combines the root folder and request path to get the full file path.
File Serving:
403 Forbidden: If the path is forbidden.
404 Not Found: If the file does not exist.
200 OK: If the file is found, reads its content and returns it with the appropriate MIME type.
Response Construction: Constructs the HTTP response headers and body based on the file type and availability.
Response Sending: Writes the HTTP response to the TCP stream and flushes it.