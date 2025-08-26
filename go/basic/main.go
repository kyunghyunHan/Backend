package main

import (
	"fmt"
	"net/http"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
)

func main() {
	r := chi.NewRouter()

	// 미들웨어
	r.Use(middleware.Logger)
	r.Use(middleware.Recoverer)

	// 라우트
	r.Get("/", func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte("Hello World!"))
	})

	r.Get("/users/{id}", func(w http.ResponseWriter, r *http.Request) {
		userID := chi.URLParam(r, "id")
		w.Write([]byte(fmt.Sprintf("User ID: %s", userID)))
	})

	// 서버 시작
	fmt.Println("Server starting on :8080")
	http.ListenAndServe(":8080", r)
}
