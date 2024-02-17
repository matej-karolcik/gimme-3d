package main

import (
	"client/pkg/request"
	"flag"
	"fmt"
	"github.com/h2non/bimg"
	_ "github.com/kolesa-team/go-webp/decoder"
	"github.com/panjf2000/ants/v2"
	_ "image/jpeg"
	_ "image/png"
	"io"
	"log"
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
		glbDir := "../glb"
		models, err := os.ReadDir(glbDir)
		if err != nil {
			return fmt.Errorf("reading glb directory: %w", err)
		}

		for _, model := range models {
			if !strings.HasSuffix(model.Name(), ".glb") {
				continue
			}
			wg.Add(1)

			if err = pool.Invoke(path.Join(glbDir, model.Name())); err != nil {
				return fmt.Errorf("invoking pool: %w", err)
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
			return fmt.Errorf("invoking pool: %w", err)
		}
	}

	wg.Wait()
	pool.Release()

	return nil
}

func handle(payload interface{}) {
	defer wg.Done()

	modelPath := payload.(string)

	modelBytes, err := os.ReadFile(modelPath)
	if err != nil {
		panic(err)
	}

	req, err := request.Create(
		endpointUrl, "", *outputFormat,
		modelBytes, map[int][]byte{0: imageBytes},
		*size, *size,
	)
	if err != nil {
		panic(err)
	}

	start := time.Now()

	resp, err := client.Do(req)
	if err != nil {
		log.Fatalln(err)
	}

	if resp.StatusCode != http.StatusOK {
		fmt.Printf("%-50serror\n", path.Base(modelPath))
		return
	}

	lock.Lock()
	if *all {
		fmt.Printf("%-50s%s\n", path.Base(modelPath), time.Since(start))
	} else {
		fmt.Printf("roundtrip time: %s\n", time.Since(start))
	}
	lock.Unlock()

	if *save {
		f, err := os.Create(path.Join(results, path.Base(
			strings.ReplaceAll(modelPath, ".glb", "."+*outputFormat))))
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
