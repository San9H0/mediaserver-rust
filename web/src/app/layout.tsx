import "../styles/globals.css";
import React, { ReactNode } from "react";
import Link from "next/link";

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body className="bg-gray-100 text-gray-800">
        {/* 헤더 */}
        <header className="bg-blue-300 text-white">
          <div className="container mx-auto flex justify-between items-center py-4 px-6">
            <h1 className="text-2xl font-bold">My Media Server</h1>
            <nav>
              <ul className="flex space-x-4">
                <li>
                  <Link href="/" className="hover:underline">
                    Home
                  </Link>
                </li>
                <li>
                  <Link href="/webrtc-broadcast" className="hover:underline">
                    WebRTC 방송
                  </Link>
                </li>
                <li>
                  <Link href="/webrtc-watch" className="hover:underline">
                    WebRTC 시청
                  </Link>
                </li>
                <li>
                  <Link href="/hls-watch" className="hover:underline">
                    HLS 시청
                  </Link>
                </li>
              </ul>
            </nav>
          </div>
        </header>

        {/* 메인 콘텐츠 */}
        <main className="container mx-auto p-6">{children}</main>

        {/* 푸터 */}
        <footer className="bg-gray-800 text-white text-center py-4">
          <p>© 2024 San9H0 Media Server. All rights reserved.</p>
        </footer>
      </body>
    </html>
  );
}
