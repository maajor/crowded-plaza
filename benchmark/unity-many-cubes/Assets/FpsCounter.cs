using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.UI;

public class FpsCounter : MonoBehaviour
{
    // Start is called before the first frame update
    public Text FpsText;
    private float _lastUpdateTime;
    private int _counter;
    void Start()
    {
        FpsText = GetComponent<Text>();
    }

    // Update is called once per frame
    void Update()
    {
        if(Time.time - _lastUpdateTime > 1.0f)
        {
            float fps = _counter/(Time.time - _lastUpdateTime);
            FpsText.text = fps.ToString();
            _counter = 0;
            _lastUpdateTime = Time.time;
        }
        _counter++;
    }
}
