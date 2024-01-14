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

	if err = addField(writer, "model", modelUrl); err != nil {
		return fmt.Errorf("adding model field: %w", err)
	}
	if err = addField(writer, "width", "2000"); err != nil {
		return fmt.Errorf("adding width field: %w", err)
	}
	if err = addField(writer, "height", "2000"); err != nil {
		return fmt.Errorf("adding height field: %w", err)
	}

	field, err := writer.CreateFormFile("textures[1]", "canvas.jpg")
	if err != nil {
		return fmt.Errorf("creating texture form file: %w", err)
	}
	if _, err = io.Copy(field, f); err != nil {
		return fmt.Errorf("writing texture file: %w", err)
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

	f, err = os.Create("output.png")
	if err != nil {
		return fmt.Errorf("creating output file: %w", err)
	}

	if _, err = io.Copy(f, resp.Body); err != nil {
		return fmt.Errorf("writing output file: %w", err)
	}

	return nil
}

func addField(writer *multipart.Writer, name, content string) error {
	field, err := writer.CreateFormField(name)
	if err != nil {
		return fmt.Errorf("creating model form field: %w", err)
	}
	if _, err = field.Write([]byte(content)); err != nil {
		return fmt.Errorf("writing model url: %w", err)
	}

	return nil
}
