# Vibe Mode Writing Training Tool - Project Brief

## Overview

This project is a web-based creative writing training tool inspired by the "Vibe Mode" functionality from the Ghostwriter desktop application. It facilitates a collaborative writing exercise between the user and an AI language model (LLM), designed to enhance writing skills through iterative, timed contributions.

## Core Concept

The essence of Vibe Mode is a dynamic back-and-forth interaction between the user and the AI:

- The user writes a segment of text within a fixed time limit.
- Once the timer expires, the AI continues the text, maintaining semantic consistency and narrative flow.
- This cycle repeats, encouraging the user to develop ideas, style, and storytelling skills in a guided, interactive manner.

This approach blends human creativity with AI assistance, fostering a productive and engaging writing practice.

## Features

- **Timed Writing Sessions:** Users have a configurable timer to write their portion, promoting focused and paced writing.

- **AI Continuation:** After the user's turn, the AI generates a continuation that aligns with the existing text's style and content.

- **Vibe Mode Styles:** Users can select from various writing "vibes" or genres (e.g., Hardboiled, Fantasy, Poetry) that influence the AI's tone and style.

- **Contextual Awareness:** The AI leverages the user's previous text and optionally a curated knowledge base to produce coherent and contextually relevant continuations.

- **Web Accessibility:** The tool operates in a web browser, with a server-side backend handling AI API interactions, enabling easy access without local installations.

## Technical Considerations

- **Frontend:** A responsive web interface for text input, timer display, and AI-generated text presentation.

- **Backend:** A server application managing user sessions, timer logic, and communication with AI APIs (e.g., OpenAI, Ollama).

- **State Management:** Maintaining conversation history and context to ensure continuity across user and AI turns.

- **Customization:** Allow users to select vibe modes and adjust parameters like timer duration and AI creativity (temperature).

- **Logging & Analytics:** Optionally track writing sessions and AI responses for user feedback and improvement.

## Purpose

This tool aims to provide writers, educators, and learners with an interactive platform to practice and improve creative writing. By combining timed human input with AI-generated continuations, it encourages disciplined writing habits while inspiring creativity through AI collaboration.

## Next Steps

Use this brief as a foundation to develop the web-based Vibe Mode writing training tool. Collaborate with coding assistants or development teams to design and implement the frontend and backend components, integrating AI APIs and ensuring a seamless user experience.

---

Happy writing and creating with AI-powered Vibe Mode!
