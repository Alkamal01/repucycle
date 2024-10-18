import React from 'react';
import { Link } from 'react-router-dom';
import './Navbar.scss';

const Navbar = () => {
  return (
    <nav className="navbar">
      <h1>RepuCycle</h1>
      <ul>
        <li><Link to="/">Home</Link></li>
        <li><Link to="/quiz">Quiz</Link></li>
        <li><Link to="/challenge">Challenges</Link></li>
      </ul>
    </nav>
  );
};

export default Navbar;
