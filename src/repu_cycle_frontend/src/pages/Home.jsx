import React from 'react';
import './Home.scss';
import logo from '../assets/logo.svg';

const Home = () => {
  return (
    <div className="home">
      <header className="home-header">
        <img src={logo} className="home-logo" alt="logo" />
        <h1>Welcome to RepuCycle</h1>
        <p>Promoting Sustainable Waste Management with Tokenized Incentives.</p>
        <button className="get-started">Get Started</button>
      </header>
    </div>
  );
};

export default Home;
