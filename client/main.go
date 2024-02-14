package main

import (
	"bytes"
	"flag"
	"fmt"
	"github.com/h2non/bimg"
	_ "github.com/kolesa-team/go-webp/decoder"
	"github.com/panjf2000/ants/v2"
	_ "image/jpeg"
	_ "image/png"
	"io"
	"log"
	"mime"
	"mime/multipart"
	"net/http"
	"os"
	"path"
	"strings"
	"sync"
	"time"
)

var (
	conc         = flag.Int("conc", 1, "number of concurrent requests")
	numRequests  = flag.Int("n", 1, "number of requests")
	all          = flag.Bool("all", false, "run all models")
	save         = flag.Bool("save", false, "save image")
	size         = flag.Int("size", 2000, "size of the preview")
	inputFormat  = flag.String("iformat", "jpg", "input format")
	outputFormat = flag.String("oformat", "png", "output format")

	lock   = &sync.Mutex{}
	wg     = &sync.WaitGroup{}
	client = &http.Client{}

	imageBytes []byte
)

const (
	results     = "out"
	endpointUrl = "http://localhost:3030/render-form"
)

func main() {
	flag.Parse()
	if err := run(); err != nil {
		panic(err)
	}
}

func run() error {
	if *all {
		fmt.Printf("running all models with %d concurrent requests, size: %d\n\n", *conc, *size)
	} else {
		fmt.Printf("running %d requests with %d concurrent requests, size: %d\n\n", *numRequests, *conc, *size)
	}

	imagePath := fmt.Sprintf("../testdata/image.%s", *inputFormat)
	//imagePath = "../testdata/canvas.png"

	if err := loadImage(imagePath); err != nil {
		return fmt.Errorf("loading image: %w", err)
	}

	if *save {
		_ = os.Mkdir(results, os.ModePerm)
	}

	pool, err := ants.NewPoolWithFunc(*conc, handle)

	if err != nil {
		return fmt.Errorf("creating pool: %w", err)
	}

	if *all {
		start := time.Now()
		models, err := os.ReadDir("../glb")
		if err != nil {
			return fmt.Errorf("reading glb directory: %w", err)
		}

		for _, model := range models {
			if !strings.HasSuffix(model.Name(), ".glb") {
				continue
			}
			wg.Add(1)

			modelUrl := "https://jq-staging-matko.s3.eu-central-1.amazonaws.com/gltf/" + model.Name()
			if err = pool.Invoke(modelUrl); err != nil {
				panic(err)
			}
		}

		wg.Wait()
		pool.Release()

		fmt.Printf("\n%-50s%s\n", "total time", time.Since(start))

		return nil
	}
	for i := 0; i < *numRequests; i++ {
		wg.Add(1)

		//const modelUrl = "https://jq-staging-matko.s3.eu-central-1.amazonaws.com/gltf/1_p1_duvet-cover_1350x2000.glb"
		const modelUrl = "https://jq-staging-matko.s3.eu-central-1.amazonaws.com/gltf/1_p1_t-shirt.glb"
		if err = pool.Invoke(modelUrl); err != nil {
			panic(err)
		}
	}

	wg.Wait()
	pool.Release()

	return nil
}

func handle(payload interface{}) {
	defer wg.Done()

	modelUrl := payload.(string)

	req, err := createRequest(endpointUrl, modelUrl)
	if err != nil {
		panic(err)
	}

	start := time.Now()

	resp, err := client.Do(req)
	if err != nil {
		log.Fatalln(err)
	}

	if resp.StatusCode != http.StatusOK {
		fmt.Printf("%-50serror\n", path.Base(modelUrl))
		return
	}

	lock.Lock()
	if *all {
		fmt.Printf("%-50s%s\n", path.Base(modelUrl), time.Since(start))
	} else {
		fmt.Printf("roundtrip time: %s\n", time.Since(start))
	}
	lock.Unlock()

	if *save {
		f, err := os.Create(path.Join(results, path.Base(
			strings.ReplaceAll(modelUrl, ".glb", "."+*outputFormat))))
		if err != nil {
			panic(err)
		}
		defer func() {
			_ = f.Close()
		}()
		if _, err = io.Copy(f, resp.Body); err != nil {
			panic(err)
		}
	}
}

func createRequest(endpointUrl, modelUrl string) (*http.Request, error) {
	buf := new(bytes.Buffer)
	writer := multipart.NewWriter(buf)

	if err := addField(writer, "model", modelUrl); err != nil {
		return nil, fmt.Errorf("adding model field: %w", err)
	}
	if err := addField(writer, "width", "3000"); err != nil {
		return nil, fmt.Errorf("adding width field: %w", err)
	}
	if err := addField(writer, "height", "2700"); err != nil {
		return nil, fmt.Errorf("adding height field: %w", err)
	}

	field, err := writer.CreateFormFile("textures[1]", "canvas.jpg")
	if err != nil {
		return nil, fmt.Errorf("creating texture form file: %w", err)
	}

	reader := bytes.NewReader(imageBytes)
	if _, err = io.Copy(field, reader); err != nil {
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
	req.Header.Add("Accept", mime.TypeByExtension("."+*outputFormat))

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

func loadImage(path string) error {
	f, err := os.Open(path)
	if err != nil {
		return fmt.Errorf("opening image: %w", err)
	}
	defer func() {
		_ = f.Close()
	}()

	imageBytes, err = bimg.Read(path)
	if err != nil {
		return fmt.Errorf("decoding image: %w", err)
	}
	im := bimg.NewImage(imageBytes)

	b, err := im.Thumbnail(*size)
	if err != nil {
		return fmt.Errorf("resizing image: %w", err)
	}

	imageBytes = b
	return nil
}
