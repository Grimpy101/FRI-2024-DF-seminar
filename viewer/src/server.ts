import express from 'express';
import path from 'path';

const app = express();
const port = 3000;

app.get('/', (request, response) => {
    response.sendFile(path.join(__dirname, '../html/index.html'))
});

app.use(express.static('static'));
app.use(express.static('dist/static'))

app.listen(port, () => {
    return console.log(`Express server at localhost:${port}`);
});