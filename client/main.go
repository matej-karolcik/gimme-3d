package main

import (
	"bytes"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"os"
)

func main() {
	if err := run(); err != nil {
		panic(err)
	}
}

func run() error {
	url := "http://localhost:3030/render-form"

	modelUrl := "https://jq-staging-matko.s3.eu-central-1.amazonaws.com/gltf/1_p1_duvet-cover_1350x2000.glb"

	client := &http.Client{}

	f, err := os.Open("../testdata/image.jpg")
	if err != nil {
		return fmt.Errorf("reading canvas file: %w", err)
	}

	buf := new(bytes.Buffer)
	writer := multipart.NewWriter(buf)

	field, err := writer.CreateFormField("model")
	if err != nil {
		return fmt.Errorf("creating model form field: %w", err)
	}
	if _, err = field.Write([]byte(modelUrl)); err != nil {
		return fmt.Errorf("writing model url: %w", err)
	}

	field, err = writer.CreateFormFile("canvas[1]", "canvas.jpg")
	if err != nil {
		return fmt.Errorf("creating canvas form file: %w", err)
	}
	if _, err = io.Copy(field, f); err != nil {
		return fmt.Errorf("writing canvas file: %w", err)
	}

	if err = writer.Close(); err != nil {
		return fmt.Errorf("closing writer: %w", err)
	}

	req, err := http.NewRequest(http.MethodPost, url, buf)
	if err != nil {
		return fmt.Errorf("creating request: %w", err)
	}

	req.Header.Add("Content-Type", writer.FormDataContentType())

	resp, err := client.Do(req)
	if err != nil {
		return fmt.Errorf("making request: %w", err)
	}

	b, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("reading response body: %w", err)
	}

	fmt.Printf("resp: %s\n", string(b))
	return nil
}
