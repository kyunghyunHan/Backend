// server.js
const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

// 미들웨어: JSON 요청 파싱
app.use(express.json());

// 기본 라우트
app.get('/', (req, res) => {
  res.send('Hello from server.js!');
});

// 서버 실행
app.listen(PORT, () => {
  console.log(`Server is running on http://localhost:${PORT}`);
});
