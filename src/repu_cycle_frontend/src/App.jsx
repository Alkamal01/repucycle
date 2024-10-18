import { BrowserRouter as Router, Route, Routes } from 'react-router-dom';
import Navbar from './components/Navbar';
import Footer from './components/Footer';
import Home from './pages/Home';
import Quiz from './pages/Quiz';
import Challenge from './pages/Challenge';

function App() {
  return (
    <Router>
      <div className="App">
        <Navbar />
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/quiz" element={<Quiz />} />
          <Route path="/challenge" element={<Challenge />} />
        </Routes>
        <Footer />
      </div>
    </Router>
  );
}

export default App;
