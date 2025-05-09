const express = require('express');
const path = require('path');

const app = express();
const PORT = 2560; // Port for the server

// Directory to serve files from
const FILES_DIR = path.join(__dirname, 'files');

// Serve static files from the specified directory

// Root route to display a message
app.get('/', (req, res) => {
    res.send('File server is running. Access files at http://localhost:' + PORT + '/<filename>');
});

app.get('/download', (req, res) => {
    console.log("\x1b[31mnodejs:\x1b[32m Download triggered\x1b[0m");
    res.sendFile(path.join(FILES_DIR, 'test.txt'), (err) => {
        if (err) {
            console.error("\x1b[31mnodejs:\x1b[32m Error sending file\x1b[0m", err);
            res.status(err.status).end();
        } else {
            console.log("\x1b[31mnodejs:\x1b[32m File sent successfully\x1b[0m");
        }
    })
})

app.get('/404', (req, res) => {
    console.log("\x1b[31mnodejs:\x1b[32m 404 error triggered\x1b[0m");
    res.status(404).send('404 Not Found');
})

// Start the server
app.listen(PORT, () => {
    console.log('\x1b[31mnodejs:\x1b[32m Server is running at http://localhost:' + PORT + '\x1b[0m');
});