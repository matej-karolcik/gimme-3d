package main

import (
	"bytes"
	"flag"
	"fmt"
	"github.com/panjf2000/ants/v2"
	"io"
	"mime"
	"mime/multipart"
	"net/http"
	"os"
	"sync"
	"time"
)

var (
	conc        = flag.Int("conc", 1, "number of concurrent requests")
	numRequests = flag.Int("n", 1, "number of requests")
	webp        = flag.Bool("webp", false, "use webp as output format")
)

func main() {
	flag.Parse()
	if err := run(); err != nil {
		panic(err)
	}
}

func run() error {
	endpointUrl := "http://localhost:3030/render-form"
	modelUrl := "https://jq-staging-matko.s3.eu-central-1.amazonaws.com/gltf/1_p1_duvet-cover_1350x2000.glb"
	client := &http.Client{}

	lock := &sync.Mutex{}
	wg := &sync.WaitGroup{}

	pool, err := ants.NewPoolWithFunc(*conc, func(payload interface{}) {
		defer wg.Done()
		start := time.Now()

		req, err := createRequest(endpointUrl, modelUrl, "../testdata/image.jpg")
		if err != nil {
			panic(err)
		}

		_, err = client.Do(req)
		if err != nil {
			panic(err)
		}

		lock.Lock()
		defer lock.Unlock()
		fmt.Printf("roundtrip time: %s\n", time.Since(start))
	})

	if err != nil {
		return fmt.Errorf("creating pool: %w", err)
	}

	for i := 0; i < *numRequests; i++ {
		wg.Add(1)
		if err = pool.Invoke(struct{}{}); err != nil {
			panic(err)
		}
	}

	wg.Wait()
	pool.Release()

	return nil
}

func createRequest(endpointUrl, modelUrl, imagePath string) (*http.Request, error) {
	buf := new(bytes.Buffer)
	writer := multipart.NewWriter(buf)

	if err := addField(writer, "model", modelUrl); err != nil {
		return nil, fmt.Errorf("adding model field: %w", err)
	}
	if err := addField(writer, "width", "2000"); err != nil {
		return nil, fmt.Errorf("adding width field: %w", err)
	}
	if err := addField(writer, "height", "2000"); err != nil {
		return nil, fmt.Errorf("adding height field: %w", err)
	}

	f, err := os.Open(imagePath)
	if err != nil {
		return nil, fmt.Errorf("reading canvas file: %w", err)
	}
	defer func() {
		_ = f.Close()
	}()

	field, err := writer.CreateFormFile("textures[1]", "canvas.jpg")
	if err != nil {
		return nil, fmt.Errorf("creating texture form file: %w", err)
	}
	if _, err = io.Copy(field, f); err != nil {
		return nil, fmt.Errorf("writing texture file: %w", err)
	}

	if err = writer.Close(); err != nil {
		return nil, fmt.Errorf("closing writer: %w", err)
	}

	req, err := http.NewRequest(http.MethodPost, endpointUrl, buf)
	if err != nil {
		return nil, fmt.Errorf("creating request: %w", err)
	}

	req.Header.Add("Content-Type", writer.FormDataContentType())

	if *webp {
		req.Header.Add("Accept", mime.TypeByExtension(".webp"))
	}

	return req, nil
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
