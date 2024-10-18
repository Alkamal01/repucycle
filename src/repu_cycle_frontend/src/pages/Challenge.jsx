import React, { useState } from 'react';
import './Challenge.scss';

const Challenge = () => {
  const [challenges] = useState([
    { id: 1, description: "Recycle 10kg of plastic", reward: 50 },
    { id: 2, description: "Collect and segregate organic waste", reward: 30 },
  ]);

  return (
    <div className="challenge-page">
      <h2>Challenges</h2>
      <ul>
        {challenges.map(challenge => (
          <li key={challenge.id}>
            {challenge.description} - Earn {challenge.reward} tokens
            <button>Accept Challenge</button>
          </li>
        ))}
      </ul>
    </div>
  );
};

export default Challenge;
