package request

import (
	"bytes"
	"fmt"
	"io"
	"mime"
	"mime/multipart"
	"net/http"
	"strconv"
)

func Create(
	endpointUrl, modelUrl, outputFormat string,
	modelBytes []byte,
	imagesMap map[int][]byte,
	width, height int,
) (*http.Request, error) {
	buf := new(bytes.Buffer)
	writer := multipart.NewWriter(buf)

	if modelUrl != "" {
		if err := addField(writer, "model_url", modelUrl); err != nil {
			return nil, fmt.Errorf("adding model field: %w", err)
		}
	}

	if modelBytes != nil {
		field, err := writer.CreateFormFile("model", "model.json")
		if err != nil {
			return nil, fmt.Errorf("creating model form file: %w", err)
		}
		if _, err = field.Write(modelBytes); err != nil {
			return nil, fmt.Errorf("writing model file: %w", err)
		}

	}
	if err := addField(writer, "width", strconv.Itoa(width)); err != nil {
		return nil, fmt.Errorf("adding width field: %w", err)
	}
	if err := addField(writer, "height", strconv.Itoa(height)); err != nil {
		return nil, fmt.Errorf("adding height field: %w", err)
	}

	for i, imageBytes := range imagesMap {
		field, err := writer.CreateFormFile(
			fmt.Sprintf("textures[%d]", i),
			fmt.Sprintf("canvas-%d.jpg", i),
		)
		if err != nil {
			return nil, fmt.Errorf("creating texture form file: %w", err)
		}

		reader := bytes.NewReader(imageBytes)
		if _, err = io.Copy(field, reader); err != nil {
			return nil, fmt.Errorf("writing texture file: %w", err)
		}
	}

	if err := writer.Close(); err != nil {
		return nil, fmt.Errorf("closing writer: %w", err)
	}

	req, err := http.NewRequest(http.MethodPost, endpointUrl, buf)
	if err != nil {
		return nil, fmt.Errorf("creating request: %w", err)
	}

	req.Header.Add("Content-Type", writer.FormDataContentType())
	req.Header.Add("Accept", mime.TypeByExtension("."+outputFormat))

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
