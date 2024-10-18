import React, { useState } from 'react';
import './Quiz.scss';

const Quiz = () => {
  const [answers, setAnswers] = useState([]);
  
  const submitQuiz = () => {
    // Logic for submitting answers
    console.log("Quiz submitted with answers: ", answers);
  };

  return (
    <div className="quiz-page">
      <h2>Waste Management Quiz</h2>
      <form onSubmit={submitQuiz}>
        {/* Example question */}
        <label>1. What is the most recyclable material?</label>
        <input
          type="text"
          onChange={(e) => setAnswers([...answers, e.target.value])}
        />
        <button type="submit">Submit Quiz</button>
      </form>
    </div>
  );
};

export default Quiz;
