import React from 'react';
import Image from 'next/image';

export default function Home() {
  return (
    <div className="container">
      <main>
        <h1>Welcome to Next.js!</h1>
        <Image src="/next.svg" alt="Next.js logo" width={180} height={38} priority />
      </main>
    </div>
  );
}